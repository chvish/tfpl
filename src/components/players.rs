use std::ops::Index;
use super::{Component, Frame};
use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use tokio::sync::mpsc::UnboundedSender;
use crate::{
  action::Action,
  config::{Config, KeyBindings},
};
use crate::components::player_card::PlayerCard;

#[derive(Default)]
pub struct Players {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
    category: String,
    players: Vec<PlayerCard>
}

impl Players {
    pub fn new(category: String) -> Self {
        Players{
            command_tx: None,
            config: Default::default(),
            category: category,
            players: vec![
                PlayerCard::new("Player A".to_string(), "Team A".to_string(), 10),
                PlayerCard::new("Player A".to_string(), "Team A".to_string(), 10),
                PlayerCard::new("Player A".to_string(), "Team A".to_string(), 10),
                PlayerCard::new("Player A".to_string(), "Team A".to_string(), 10),
            ]
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
    match action {
      Action::Tick => {
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
      let player_layout = Layout::default().direction(Direction::Horizontal).constraints([
          Constraint::Percentage(25),
          Constraint::Percentage(25),
          Constraint::Percentage(25),
          Constraint::Percentage(25),
      ]).split(area);
      for i in 0..self.players.len() {
          self.players[i].draw(f, player_layout[i]);
      }
      f.render_widget(Block::new().borders(Borders::ALL).title(self.category.clone()).title_style(Style::new().bg(Color::LightBlue)), area);
    Ok(())
  }
}
