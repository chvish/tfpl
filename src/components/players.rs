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

#[derive(Default)]
pub struct Players {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
    category: String,
}

impl Players {
    pub fn new(category: String) -> Self {
        Players{
            command_tx: None,
            config: Default::default(),
            category: category,
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
      f.render_widget(Block::new().borders(Borders::ALL).title(self.category.clone()).title_style(Style::new().bg(Color::LightBlue)), area);
    Ok(())
  }
}
