use color_eyre::eyre::Result;
use fpl_api::bootstrap::Element;
use image::DynamicImage;
use ratatui::{prelude::*, widgets::*};
use ratatui_image::{
    picker::Picker,
    protocol::{Protocol, StatefulProtocol},
    Image, StatefulImage,
};
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
    pub details: Element,
    is_active: bool,
    image_picker: Option<Picker>,
    image_state: Option<StatefulProtocol>,
    team_image_state: Option<StatefulProtocol>,
    pub position: i64,
    debug: Vec<u8>,
}

impl PlayerCard {
    pub fn new(
        name: String,
        team: String,
        details: Element,
        image_picker: Option<Picker>,
        team_image_state: Option<StatefulProtocol>,
        position: i64,
    ) -> Self {
        PlayerCard {
            command_tx: None,
            config: Default::default(),
            name,
            team,
            details,
            is_active: false,
            image_picker,
            image_state: None,
            team_image_state,
            position,
            debug: Vec::new(),
        }
    }

    pub fn mark_active(&mut self, state: bool) {
        self.is_active = state;
    }

    pub fn set_image(&mut self, image: DynamicImage) {
        if let Some(pc) = self.image_picker.as_mut() {
            let protocol = pc.new_resize_protocol(image);
            self.image_state = Some(protocol);
        }
    }

    pub fn has_image(&self) -> bool {
        // TODO: figure out why state is being shared
        // false
        self.image_state.is_some()
    }
}

impl PlayerCard {
    pub fn draw_big(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layouts = Layout::default()
            .constraints([Constraint::Length(1), Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(Direction::Vertical)
            .split(area);
        let image_layput = Layout::default()
            .constraints([Constraint::Fill(1), Constraint::Length(12), Constraint::Fill(1)])
            .direction(Direction::Horizontal)
            .split(layouts[1])[1];
        f.render_widget(Clear, area);
        // TODO
        let block = Block::default().borders(Borders::ALL).border_set(symbols::border::DOUBLE);
        let p = Paragraph::new(vec![
            Line::styled(self.name.to_string(), Style::default().bg(Color::Indexed(127_u8)).fg(Color::White)),
            Line::raw(self.team.clone()),
            Line::from(format!("Points: {}", self.details.event_points)),
            Line::from(format!("Total Goals: {}", self.details.goals_scored)),
            Line::from(format!("Total Assists: {}", self.details.assists)),
            Line::from(format!("EP this: {}", self.details.ep_this)),
            Line::from(format!("EP next : {}", self.details.ep_next)),
            Line::from(format!("Bonus: {}", self.details.bonus)),
        ])
        .alignment(Alignment::Center);

        f.render_widget(p, layouts[2]);
        f.render_widget(block, area);

        if let Some(image) = self.image_state.as_mut() {
            let s_image = StatefulImage::new(None).resize(ratatui_image::Resize::Crop(None));
            StatefulWidget::render(s_image, image_layput, f.buffer_mut(), image);
            // f.render_stateful_widget(s_image, image_layput, image);
        }

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
        action == Action::Tick;
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layouts = Layout::default()
            .constraints([Constraint::Length(1), Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(Direction::Vertical)
            .split(area);
        let image_layput = Layout::default()
            .constraints([Constraint::Fill(1), Constraint::Length(6), Constraint::Fill(1)])
            .direction(Direction::Horizontal)
            .split(layouts[1])[1];
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
            .border_style(Style::default().fg(Color::Indexed(color_idx)));
        let mut name_details =
            vec![Span::styled(self.name.clone(), Style::default().bg(Color::Indexed(127_u8)).fg(Color::White))];
        match self.details.status.as_str() {
            "i" => name_details.push(Span::from("🚩")),
            "d" => name_details.push(Span::from("⚠️")),
            _ => {},
        };

        let p = Paragraph::new(vec![
            Line::from(name_details),
            // Line::raw(self.team.clone()),
            Line::from(format!("Points: {}", self.details.event_points)),
        ])
        .alignment(Alignment::Center)
        .block(b);

        f.render_widget(p, layouts[2]);
        if let Some(image) = self.team_image_state.as_mut() {
            let s_image = StatefulImage::new(None).resize(ratatui_image::Resize::Crop(None));
            f.render_stateful_widget(s_image, image_layput, image);
        }

        Ok(())
    }
}
