use std::collections::HashMap;

use bytes::Bytes;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use fpl_api;
use image::{DynamicImage, ImageReader};
use ratatui::prelude::Rect;
use ratatui_image::{
    picker::{Picker, ProtocolType},
    protocol::StatefulProtocol,
    StatefulImage,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, mpsc::UnboundedSender};

use crate::{
    action::Action,
    components::{fps::FpsCounter, home::Home, Component},
    config::Config,
    event::Event,
    mode::Mode,
    tui,
};

pub struct App {
    pub config: Config,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub components: Vec<Box<dyn Component>>,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub mode: Mode,
    pub last_tick_key_events: Vec<KeyEvent>,
    fpl_client: fpl_api::FPLClient,
    bootstrap_data: fpl_api::bootstrap::BootstrapData,
}

fn decode_bytes_to_image(data: Bytes) -> Result<DynamicImage, image::ImageError> {
    ImageReader::new(std::io::Cursor::new(data)).with_guessed_format()?.decode()
}

async fn get_player_image(pc: i64, tx: UnboundedSender<Event>) {
    let resp =
        reqwest::get(format!("https://resources.premierleague.com/premierleague/photos/players/110x140/p{}.png", pc))
            .await;
    if let Ok(ok_resp) = resp {
        if let Ok(bytes) = ok_resp.bytes().await {
            if let Ok(image) = decode_bytes_to_image(bytes) {
                let _ = tx.send(Event::PlayerImage(pc, image));
            }
        }
    }
}

#[cfg(unix)]
fn get_picker() -> Option<Picker> {
    Picker::from_query_stdio().ok()
}

#[cfg(target_os = "windows")]
fn get_picker() -> Option<Picker> {
    use windows_sys::Win32::{System::Console::GetConsoleWindow, UI::HiDpi::GetDpiForWindow};

    struct FontSize {
        pub width: u16,
        pub height: u16,
    }
    impl Default for FontSize {
        fn default() -> Self {
            FontSize { width: 17, height: 38 }
        }
    }

    let size: FontSize = match unsafe { GetDpiForWindow(GetConsoleWindow()) } {
        96 => FontSize { width: 9, height: 20 },
        120 => FontSize { width: 12, height: 25 },
        144 => FontSize { width: 14, height: 32 },
        _ => FontSize::default(),
    };

    let mut picker = Picker::new((size.width, size.height));

    let protocol = picker.guess_protocol();

    if protocol == ProtocolType::Halfblocks {
        return None;
    }
    Some(picker)
}

impl App {
    pub async fn new(tick_rate: f64, frame_rate: f64, player_id: String) -> Result<Self> {
        let fps = FpsCounter::default();
        let config = Config::new()?;
        let mode = Mode::Home;
        let fpl_client = fpl_api::FPLClient::new();
        let bootstrap_data = fpl_client.get_bootstrap_data().await?;
        let manager = fpl_client.get_manager_details(&player_id).await?;
        let fixtures = fpl_client.get_fixtures().await?;
        let gw_picks = fpl_client.get_manager_team_for_gw(&player_id, &manager.current_event.to_string()).await?;
        let ti = Self::load_team_images().await?;
        let home = Home::new(manager, bootstrap_data.clone(), gw_picks, fixtures, get_picker(), ti);
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(home)],
            should_quit: false,
            should_suspend: false,
            config,
            mode,
            last_tick_key_events: Vec::new(),
            fpl_client,
            bootstrap_data,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new(event_tx.clone())?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(action_tx.clone())?;
        }

        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }

        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        loop {
            if let Some(e) = event_rx.recv().await {
                match e {
                    Event::Quit => action_tx.send(Action::Quit)?,
                    Event::Tick => action_tx.send(Action::Tick)?,
                    Event::Render => action_tx.send(Action::Render)?,
                    Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    Event::Key(key) => {
                        if let KeyCode::Char('q') = key.code { action_tx.send(Action::Quit)? }
                    },
                    _ => {},
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.handle_events(Some(e.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    },
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::GetPlayerImage(player_code) => {
                        let task_event = event_tx.clone();
                        tokio::spawn(get_player_image(player_code, task_event));
                    },
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.area());
                                if let Err(e) = r {
                                    action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                                }
                            }
                        })?;
                    },
                    Action::Render => {
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.area());
                                if let Err(e) = r {
                                    action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                                }
                            }
                        })?;
                    },
                    _ => {},
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.update(action.clone())? {
                        action_tx.send(action)?
                    };
                }
            }
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                tui = tui::Tui::new(event_tx.clone())?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn load_team_images() -> Result<HashMap<i64, DynamicImage>> {
        let mut ti = HashMap::new();
        for i in 1..100 {
            match ImageReader::open(format!("./assets/t{}@x2.png", i)).ok() {
                None => (),
                Some(imf) => {
                    let dy = imf.decode()?;
                    ti.insert(i, dy);
                },
            }
        }
        Ok(ti)
    }
}
