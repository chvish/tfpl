use std::ops::Index;

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    components::player_card::PlayerCard,
    config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct Players {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    category: String,
    // TODO: better api
    pub players: Vec<PlayerCard>,
}

impl Players {
    pub fn new(category: String, players: Vec<PlayerCard>) -> Self {
        Players { command_tx: None, config: Default::default(), category, players }
    }
}

impl Players {
    fn get_contraints_and_start_pos(&self, num_player: usize) -> (Vec<Constraint>, usize) {
        match num_player {
            1 | 3 | 5 => {
                (
                    vec![
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                    ],
                    match num_player {
                        5 => 0,
                        3 => 1,
                        _ => 2,
                    },
                )
            },
            _ => {
                (
                    vec![
                        Constraint::Percentage(10),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(10),
                    ],
                    match num_player {
                        4 => 1,
                        _ => 2,
                    },
                )
            },
        }
    }
}

impl Component for Players {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        action == Action::Tick;
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let design = self.get_contraints_and_start_pos(self.players.len());
        let player_layout =
            Layout::default().direction(Direction::Horizontal).constraints(design.0).margin(1).split(area);
        for i in 0..self.players.len() {
            let given_by_layput = player_layout[design.1 + i];
            let pa = Rect::new(given_by_layput.x, area.y, given_by_layput.width, area.height);
            self.players[i].draw(f, pa)?;
        }
        Ok(())
    }
}
