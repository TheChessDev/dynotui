use color_eyre::Result;

use ratatui::layout::{Constraint, Layout};
use ratatui::style::palette::tailwind::INDIGO;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};
use ratatui::Frame;
use ratatui::{layout::Rect, style::Style};

use crate::action::Action;

use super::Component;

#[derive(Default)]
pub struct StatusBox {
    pub status_text: String,
}

impl StatusBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_status_text(&mut self, new_status: &str) {
        self.status_text = new_status.to_string();
    }
}

impl Component for StatusBox {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::UpdateStatusText(text) => self.set_status_text(&text),
            _ => {}
        };

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [_, right] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(area);
        let [_, bottom] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(right);

        Paragraph::new(self.status_text.clone())
            .block(Block::default().padding(Padding::horizontal(2)))
            .style(Style::new().fg(INDIGO.c700))
            .render(bottom, frame.buffer_mut());

        Ok(())
    }
}
