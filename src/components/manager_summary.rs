use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use tokio::sync::mpsc::UnboundedSender;
use thousands::Separable;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

pub struct ManagerSummary {
    details: fpl_api::manager::Manager,
}

impl ManagerSummary {
    pub fn new(details: fpl_api::manager::Manager) -> Self {
        Self { details }
    }

    fn get_player_flag_emoji(&self) -> &str {
        emojis::get_by_shortcode(&self.details.player_region_name.clone().to_ascii_lowercase())
            .map_or("?", |x| x.as_str())
    }

    pub fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let p = Paragraph::new(vec![
            Line::styled(self.details.name.clone(), Style::default().bg(Color::Indexed(127 as u8)).fg(Color::White)),
            Line::from(format!("({} {}, {})", &self.details.player_first_name, &self.details.player_last_name, self.get_player_flag_emoji())),
            Line::from("-------------------------"),
            Line::from(format!("Overall Rank: {}", self.details.summary_overall_rank.separate_with_commas())),
            Line::from(format!("Overall Points: {}", self.details.summary_overall_points.to_string())),
            Line::from("-------------------------"),
            Line::from(format!("GW Rank: {}", self.details.summary_event_rank.separate_with_commas())),
            Line::from(format!("GW Points: {}", self.details.summary_event_points.to_string())),
        ])
        .block(Block::default().borders(Borders::ALL).padding(Padding::new(0, 0, 5, 5)))
        .alignment(Alignment::Center);
        f.render_widget(p, area);
        Ok(())
    }
}
