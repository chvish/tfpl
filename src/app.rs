use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use fpl_api;
use ratatui::prelude::Rect;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    action::Action,
    components::{fps::FpsCounter, home::Home, Component},
    config::Config,
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

impl App {
    pub async fn new(tick_rate: f64, frame_rate: f64, player_id: String) -> Result<Self> {
        let fps = FpsCounter::default();
        let config = Config::new()?;
        let mode = Mode::Home;
        let fpl_client = fpl_api::FPLClient::new();
        let bootstrap_data = fpl_client.get_bootstrap_data().await?;
        let manager = fpl_client.get_manager_details(&player_id).await?;
        let gw_picks = fpl_client.get_manager_team_for_gw(&player_id, &manager.current_event.to_string()).await?;

        // Should use Picker::from_termios(), to get the font size,
        // but we can't put that here because that would break doctests!
        let mut picker = Picker::from_termios().unwrap();
        // Guess the protocol.
        picker.guess_protocol();
        // Load an image with the image crate.
        let dyn_img = image::io::Reader::open("./p223094.png")?.decode()?;

        // Create the Protocol which will be used by the widget.
        let mut image2 = picker.new_resize_protocol(dyn_img);

        let home = Home::new(manager, bootstrap_data.clone(), gw_picks, image2);
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(home), Box::new(fps)],
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

        let mut tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
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
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => {
                        match key.code {
                            KeyCode::Enter => action_tx.send(Action::Enter)?,
                            KeyCode::Esc => action_tx.send(Action::Escape)?,
                            KeyCode::Left => action_tx.send(Action::Left)?,
                            KeyCode::Right => action_tx.send(Action::Right)?,
                            KeyCode::Up => action_tx.send(Action::Up)?,
                            KeyCode::Down => action_tx.send(Action::Down)?,
                            KeyCode::Char('q') => action_tx.send(Action::Quit)?,
                            _ => {},
                        }
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
                        // TODO
                        ()
                    },
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
                                if let Err(e) = r {
                                    action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                                }
                            }
                        })?;
                    },
                    Action::Render => {
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
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
                tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
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
}
