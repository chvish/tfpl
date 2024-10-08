use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct PlayerCard {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    name: String,
    team: String,
    points: u32,
    is_active: bool,
}

impl PlayerCard {
    pub fn new(name: String, team: String, points: u32) -> Self {
        PlayerCard { command_tx: None, config: Default::default(), name, team, points, is_active: false }
    }
        pub fn mark_active(&mut self, state: bool) {
            self.is_active = state;
        }
}

impl Component for PlayerCard {
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
            _ => {},
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let color_idx = match self.is_active {
            true => 127u8,
            false => 255u8
        };
        let border_type = match self.is_active {
            true => BorderType::Thick,
            false => BorderType::Rounded, 
        };
        let p = Paragraph::new(vec![
            Line::styled(self.name.clone(), Style::default().bg(Color::Indexed(127 as u8)).fg(Color::White)),
            Line::raw(self.team.clone()),
            Line::from(format!("Points: {}", self.points)),
        ])
            .block(
                Block::default().borders(Borders::ALL)
                .border_type(border_type)
                .padding(Padding::new(0, 0, 1, 1))
                .border_style(
                    Style::default().fg(Color::Indexed(color_idx as u8))
                ),
        )
        .alignment(Alignment::Center);
        f.render_widget(p, area);
        Ok(())
    }
}
