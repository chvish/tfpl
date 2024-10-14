use color_eyre::eyre::Result;
use fpl_api::bootstrap::Element;
use ratatui::{prelude::*, widgets::*};
use ratatui_image::{protocol::StatefulProtocol, Image, StatefulImage};
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

pub struct PlayerCard {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    name: String,
    team: String,
    details: Element,
    is_active: bool,
    image: Box<dyn StatefulProtocol>,
}

pub struct BigPlayerCard {
    name: String,
    team: String,
    details: Element,
    image: Box<dyn StatefulProtocol>,
}

impl BigPlayerCard {
    pub fn new(name: String, team: String, details: Element, image: Box<dyn StatefulProtocol>) -> Self {
        BigPlayerCard { name, team, details, image }
    }
}

impl PlayerCard {
    pub fn new(name: String, team: String, details: Element, image: Box<dyn StatefulProtocol>) -> Self {
        PlayerCard { command_tx: None, config: Default::default(), name, team, details, is_active: false, image }
    }

    pub fn mark_active(&mut self, state: bool) {
        self.is_active = state;
    }

    pub fn get_player_big_widget(&self) -> BigPlayerCard {
        BigPlayerCard {
            name: self.name.clone(),
            team: self.team.clone(),
            details: self.details.to_owned(),
            image: self.image.clone(),
        }
    }
}

impl BigPlayerCard {
    pub fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layouts = Layout::default()
            .constraints([Constraint::Length(1), Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(Direction::Vertical)
            .split(area);
        let image_layput = Layout::default()
            .constraints([Constraint::Length(30), Constraint::Fill(1)])
            .direction(Direction::Horizontal)
            .split(layouts[1])[1];
        f.render_widget(Clear, area);
        // TODO
        let block = Block::default().borders(Borders::ALL);
        // .style(Style::new().white().on_blue());
        let p = Paragraph::new(vec![
            Line::styled(format!("{}", self.name), Style::default().bg(Color::Indexed(127 as u8)).fg(Color::White)),
            Line::raw(self.team.clone()),
            Line::from(format!("Points: {}", self.details.event_points)),
            Line::from(format!("Total Goals: {}", self.details.goals_scored)),
            Line::from(format!("Total Assists: {}", self.details.assists)),
        ])
        .alignment(Alignment::Center); //.block(block);
                                       // Render with the protocol state.
        f.render_widget(p, layouts[2]);
        f.render_widget(block, area);
        // self.image.on_blue();
        let image = StatefulImage::new(None).resize(ratatui_image::Resize::Crop(None));
        // Render with the protocol state.
        f.render_stateful_widget(image, image_layput, &mut self.image);

        // TODO
        Ok(())
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
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Min(3)])
            .split(area);
        let color_idx = match self.is_active {
            true => 127u8,
            false => 255u8,
        };
        let border_type = match self.is_active {
            true => BorderType::Thick,
            false => BorderType::Rounded,
        };
        let b = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .padding(Padding::new(0, 0, 1, 0))
            .border_style(Style::default().fg(Color::Indexed(color_idx as u8)));
        let p = Paragraph::new(vec![
            Line::styled(self.name.clone(), Style::default().bg(Color::Indexed(127 as u8)).fg(Color::White)),
            Line::raw(self.team.clone()),
            Line::from(format!("Points: {}", self.details.event_points)),
        ])
        .alignment(Alignment::Center)
        .block(b);

        f.render_widget(p, area);
        // f.render_widget(b, area);

        // TODO
        let image = StatefulImage::new(None).resize(ratatui_image::Resize::Crop(None));
        // Render with the protocol state.
        // f.render_stateful_widget(image, layouts[0], &mut self.image);

        // TODO
        Ok(())
    }
}
