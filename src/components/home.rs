
use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_big_text::{BigTextBuilder, PixelSize};

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
};
use crate::components::players::Players;

pub struct Home {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  defenders: Players,
  midfielders: Players,
  forwards: Players,
  bench: Players,
}

impl Home {
  pub fn new() -> Self {
    Home {
      command_tx: None,
      config: Default::default(),
      defenders: Players::new("DEFENDERS".to_string()),
      midfielders: Players::new("MIDFIELDERS".to_string()),
      forwards: Players::new("FORWARDS".to_string()),
      bench: Default::default(),
    }
  }
}

impl Default for Home {
    fn default() -> Self {
      Home::new()
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
      Action::Tick => {
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
      let overall_layout  = Layout::default().direction(Direction::Horizontal).constraints([
        Constraint::Percentage(20),
        Constraint::Percentage(80),
      ]).split(area);
    let banner = BigTextBuilder::default()
        .pixel_size(PixelSize::Sextant)
        .lines(vec!["Terminal".into(), "FPL".into()])
        .centered()
        .build();
      f.render_widget(banner, overall_layout[0] );
    let layouts = Layout::default().direction(Direction::Vertical)
          .constraints([
                       Constraint::Percentage(30), 
                       Constraint::Percentage(30), 
                       Constraint::Percentage(30),
                       Constraint::Percentage(10),
          ])
          .split(overall_layout[1]);
    self.defenders.draw(f, layouts[0])?;
    self.midfielders.draw(f, layouts[1])?;
    self.forwards.draw(f, layouts[2])?;
    f.render_widget(Block::new().borders(Borders::ALL).title("Bench"), layouts[3]);
    Ok(())
  }
}

