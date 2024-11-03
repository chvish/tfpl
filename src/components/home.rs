use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use fpl_api::fixture::Fixtures;
use image::DynamicImage;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_big_text::{BigTextBuilder, PixelSize};

use super::{manager_summary, Component, Frame};
use crate::{
    action::Action,
    components::{manager_summary::ManagerSummary, player_card::PlayerCard, players::Players},
    config::{Config, KeyBindings},
    event::Event,
};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,

    // TODO: do i need this? can just keep a vector of players
    picked_players: [Players; 5],
    player_code_to_player: HashMap<i64, (usize, usize)>,
    manager_summary: ManagerSummary,
    fixtures: Fixtures,

    // UI state
    active_player_coordinate: (usize, usize),
    show_player_big: bool,
}

impl Home {
    pub fn new(
        manager: fpl_api::manager::Manager,
        bootstrap_data: fpl_api::bootstrap::BootstrapData,
        gw_picks: fpl_api::manager::GWTeam,
        fixtures: Fixtures,
        mut picker: Option<Picker>,
        team_to_badge: HashMap<i64, DynamicImage>,
    ) -> Self {
        let player_id_to_details: HashMap<i64, fpl_api::bootstrap::Element> =
            bootstrap_data.elements.iter().fold(HashMap::new(), |mut m, p| {
                m.insert(p.id, p.clone());
                m
            });
        let team_id_to_details: HashMap<i64, fpl_api::bootstrap::Team> =
            bootstrap_data.teams.iter().fold(HashMap::new(), |mut m, p| {
                m.insert(p.id, p.clone());
                m
            });
        // fixtures.get(0).map(|f| f.event)
        let picked_players: (
            Vec<PlayerCard>,
            Vec<PlayerCard>,
            Vec<PlayerCard>,
            Vec<PlayerCard>,
            Vec<PlayerCard>,
            HashMap<i64, (usize, usize)>,
        ) = gw_picks.picks.iter().fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), HashMap::new()),
            |mut li, p| {
                let player_detail = player_id_to_details.get(&p.element).unwrap(); // TODO: handle. but should never happen
                let pc = PlayerCard::new(
                    format!("{} {}", player_detail.first_name, player_detail.second_name),
                    team_id_to_details.get(&player_detail.team).map(|t| t.name.clone()).unwrap(),
                    player_detail.to_owned(),
                    picker.clone(),
                    match picker.as_mut() {
                        None => None,
                        Some(p) => {
                            team_to_badge.get(&player_detail.team_code).map(|d| p.new_resize_protocol(d.clone()))
                        },
                    },
                );
                let player_code = player_detail.code;
                match (p.position, player_detail.element_type) {
                    (12 | 13 | 14 | 15, _) => {
                        li.5.insert(player_code, (4, li.4.len()));
                        li.4.push(pc);
                    },
                    (_, 1) => {
                        li.5.insert(player_code, (0, li.0.len()));
                        li.0.push(pc);
                    },
                    (_, 2) => {
                        li.5.insert(player_code, (1, li.1.len()));
                        li.1.push(pc);
                    },
                    (_, 3) => {
                        li.5.insert(player_code, (2, li.2.len()));
                        li.2.push(pc)
                    },
                    (_, 4) => {
                        li.5.insert(player_code, (3, li.3.len()));
                        li.3.push(pc)
                    },
                    _ => (),
                };
                li
            },
        );
        let active_player_coordinate = (0, 0);
        Home {
            command_tx: None,
            config: Default::default(),
            picked_players: [
                Players::new("Goalkeepers".to_string(), picked_players.0),
                Players::new("Defenders".to_string(), picked_players.1),
                Players::new("Midfielders".to_string(), picked_players.2),
                Players::new("Forwards".to_string(), picked_players.3),
                Players::new("Bench".to_string(), picked_players.4),
            ],
            player_code_to_player: picked_players.5,
            manager_summary: ManagerSummary::new(manager),
            fixtures,
            active_player_coordinate,
            show_player_big: false,
        }
    }

    fn mark_player_active_state(&mut self, coordinate: (usize, usize), state: bool) {
        self.picked_players[coordinate.0].players.get_mut(coordinate.1).map(|x| x.mark_active(state));
    }

    fn update_player_active(&mut self, old: (usize, usize)) {
        // TODO: this is  becoming a bit gnarly. would it not be better to just
        // have one state and draw that, this component thing is getting complicated
        self.mark_player_active_state(old, false);
        self.mark_player_active_state(self.active_player_coordinate, true);
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            Some(Event::PlayerImage(pc, image)) => {
                if let Some(cord) = self.player_code_to_player.get(&pc) {
                    self.picked_players[cord.0].players.get_mut(cord.1).unwrap().set_image(image.clone());
                }
                None
            },
            _ => None,
        };
        Ok(r)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Enter => {
                self.show_player_big = true;
                let coordinate = self.active_player_coordinate;
                if let Some(current_player) = self.picked_players[coordinate.0].players.get(coordinate.1) {
                    if !current_player.has_image() {
                        let pc = current_player.details.code;
                        Ok(Some(Action::GetPlayerImage(pc)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            },
            KeyCode::Esc => {
                self.show_player_big = false;
                Ok(None)
            },

            KeyCode::Left => {
                let old = self.active_player_coordinate;
                if self.active_player_coordinate.1 != 0 {
                    self.active_player_coordinate.1 -= 1;
                }
                self.update_player_active(old);
                Ok(None)
            },
            KeyCode::Right => {
                let old = self.active_player_coordinate;
                if self.picked_players[self.active_player_coordinate.0].players.len()
                    != self.active_player_coordinate.1 + 1
                {
                    self.active_player_coordinate.1 += 1;
                }
                self.update_player_active(old);
                Ok(None)
            },
            KeyCode::Up => {
                let old = self.active_player_coordinate;
                self.active_player_coordinate.0 = match self.active_player_coordinate.0 {
                    0 => 0,
                    _ => self.active_player_coordinate.0 - 1,
                };
                self.active_player_coordinate.1 = 0;
                self.update_player_active(old);
                Ok(None)
            },
            KeyCode::Down => {
                let old = self.active_player_coordinate;
                self.active_player_coordinate.0 = match self.active_player_coordinate.0 {
                    3 => 3,
                    _ => self.active_player_coordinate.0 + 1,
                };
                self.active_player_coordinate.1 = 0;
                self.update_player_active(old);
                Ok(None)
            },
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            _ => {},
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let overall_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(overall_layout[0]);
        let tagline_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(left_layout[0]);
        f.render_widget(Block::new().borders(Borders::ALL), left_layout[0]);
        let banner =
            BigTextBuilder::default().pixel_size(PixelSize::Sextant).lines(vec!["tfpl".into()]).centered().build();
        banner.render(left_layout[0], f.buffer_mut());
        // f.render_widget(banner, left_layout[0]);
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                // TODO: fix height to make bench less importatn
                Constraint::Length(1), // The 1 px here is becuase i stretch inside the nextedt layout
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Min(1),
                Constraint::Length(1), // The 1 px here is becuase i stretch inside the nextedt layout
            ])
            .split(overall_layout[1]);
        f.render_widget(Block::new().borders(Borders::ALL), overall_layout[1]);
        for i in 1..6 {
            self.picked_players[i - 1].draw(f, layouts[i])?;
        }
        self.manager_summary.draw(f, left_layout[1])?;
        // f.render_widget(Block::new().borders(Borders::ALL).title("Bench"), layouts[4]);

        if self.show_player_big {
            let card_layout =
                Layout::default().constraints([Constraint::Percentage(100)]).margin(4).split(overall_layout[1])[0];
            self.picked_players[self.active_player_coordinate.0]
                .players
                .get_mut(self.active_player_coordinate.1)
                .unwrap()
                .draw_big(f, card_layout)?;
        }
        Ok(())
    }
}
