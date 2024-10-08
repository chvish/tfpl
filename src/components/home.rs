use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_big_text::{BigTextBuilder, PixelSize};

use super::{manager_summary, Component, Frame};
use crate::{
    action::Action,
    components::{manager_summary::ManagerSummary, player_card::PlayerCard, players::Players},
    config::{Config, KeyBindings},
};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    // TODO: do i need this? can just keep a vector of players
    picked_players: [Players; 5],
    manager_summary: ManagerSummary,
    active_player_coordinate: (usize, usize)
}

impl Home {
    pub fn new(
        manager: fpl_api::manager::Manager,
        bootstrap_data: fpl_api::bootstrap::BootstrapData,
        gw_picks: fpl_api::manager::GWTeam,
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
        let picked_players: (Vec<PlayerCard>,Vec<PlayerCard>,Vec<PlayerCard>,Vec<PlayerCard>, Vec<PlayerCard>) =
            gw_picks.picks.iter().fold((Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()), |mut li, p| {
                let player_detail = player_id_to_details.get(&p.element).unwrap(); // TODO: handle. but should never happen
                let pc = PlayerCard::new(
                    format!("{} {}", player_detail.first_name, player_detail.second_name),
                    team_id_to_details.get(&player_detail.team).map(|t| t.name.clone()).unwrap(),
                    player_detail.event_points.try_into().unwrap(),
                );
                match (p.position, player_detail.element_type) {
                    (12 | 13 | 14 | 15, _) => li.4.push(pc),
                    (_, 1) => li.0.push(pc),
                    (_, 2) => li.1.push(pc),
                    (_, 3) => li.2.push(pc),
                    (_, 4) => li.3.push(pc),
                    _ => (),
                };
                li
            });
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
            manager_summary: ManagerSummary::new(manager),
            active_player_coordinate
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

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {},
            Action::Right => {
                let old = self.active_player_coordinate;
                if self.picked_players[self.active_player_coordinate.0].players.len() != self.active_player_coordinate.1 +1 {
                    self.active_player_coordinate.1 += 1;
                }
                self.update_player_active(old);
            }
            Action::Left => {
                let old = self.active_player_coordinate;
                if self.active_player_coordinate.1 != 0 { 
                    self.active_player_coordinate.1 -= 1;
                }
                self.update_player_active(old);
            }
            Action::Up => {
                let old = self.active_player_coordinate;
                self.active_player_coordinate.0 = match self.active_player_coordinate.0 {
                    0 => 0,
                    _ => self.active_player_coordinate.0 - 1
                };
                self.active_player_coordinate.1 = 0;
                self.update_player_active(old);
            }
            Action::Down => {
                let old = self.active_player_coordinate;
                self.active_player_coordinate.0 = match self.active_player_coordinate.0 {
                    3 => 3,
                    _ => self.active_player_coordinate.0 + 1
                };
                self.active_player_coordinate.1 = 0;
                self.update_player_active(old);
            }
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
        f.render_widget(banner, left_layout[0]);
        f.render_widget(Paragraph::new("FPL, in the terminal!").centered(), tagline_layout[1]);
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(overall_layout[1]);
        f.render_widget(Block::new().borders(Borders::ALL), overall_layout[1]);
        for i in 0..5 {
            self.picked_players[i].draw(f, layouts[i])?;
        }
        self.manager_summary.draw(f, left_layout[1])?;
        f.render_widget(Block::new().borders(Borders::ALL).title("Bench"), layouts[4]);
        Ok(())
    }
}
