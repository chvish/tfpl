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
    components::{manager_summary::ManagerSummary, players::Players},
    config::{Config, KeyBindings},
};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    defenders: Players,
    midfielders: Players,
    forwards: Players,
    bench: Players,
    manager_summary: ManagerSummary,
}

impl Home {
    pub fn new(manager: fpl_api::manager::Manager, bootstrap_data: fpl_api::bootstrap::BootstrapData, gw_picks: fpl_api::manager::GWTeam) -> Self {
        let m:HashMap<i64, fpl_api::bootstrap::Element>  =   bootstrap_data.elements.iter().fold(
            HashMap::new(),
            |m, p| {
                m[&p.id] = p.clone();
                m
            }
        );
        gw_picks.picks.iter().map(
            |x| {
                x.element
            }
        );
        Home {
            command_tx: None,
            config: Default::default(),
            defenders: Players::new("DEFENDERS".to_string()),
            midfielders: Players::new("MIDFIELDERS".to_string()),
            forwards: Players::new("FORWARDS".to_string()),
            bench: Default::default(),
            manager_summary: ManagerSummary::new(manager),
        }
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
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(10),
            ])
            .split(overall_layout[1]);
        self.defenders.draw(f, layouts[0])?;
        self.midfielders.draw(f, layouts[1])?;
        self.forwards.draw(f, layouts[2])?;
        self.manager_summary.draw(f, left_layout[1])?;
        f.render_widget(Block::new().borders(Borders::ALL).title("Bench"), layouts[3]);
        Ok(())
    }
}
