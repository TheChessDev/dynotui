use color_eyre::Result;
use ratatui::layout::Alignment;
use ratatui::prelude::Widget;
use ratatui::prelude::*;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use style::palette::tailwind::EMERALD;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::config::Config;

use super::Component;

#[derive(Default)]
pub struct AWSRegionBox {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    active: bool,
    region: String,
}

impl AWSRegionBox {
    pub fn new(region: &str) -> Self {
        Self {
            region: region.to_string(),
            ..Default::default()
        }
    }

    fn reset(&mut self) {
        self.active = false;
    }
}

impl Component for AWSRegionBox {
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
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [left, _] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(area);

        let [top_left, _, _] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(left);

        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("AWS Region");

        if self.active {
            block = block.border_style(Style::default().fg(EMERALD.c300));
        }

        Paragraph::new(self.region.clone())
            .alignment(Alignment::Center)
            .block(block)
            .render(top_left, frame.buffer_mut());

        Ok(())
    }
}
