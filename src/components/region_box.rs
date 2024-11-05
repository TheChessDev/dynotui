use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Alignment;
use ratatui::prelude::Widget;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::{Component, SELECTED_COLOR};

pub struct AWSRegionBox {
    pub selected: bool,
    pub region: String,
}

impl AWSRegionBox {
    pub fn new(region: &str) -> Self {
        Self {
            selected: false,
            region: region.to_string(),
        }
    }
}

impl Component for AWSRegionBox {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("AWS Region");

        if self.selected {
            block = block.border_style(Style::default().fg(SELECTED_COLOR));
        }

        Paragraph::new(self.region.clone())
            .alignment(Alignment::Center)
            .block(block)
            .render(area, buf);
    }

    fn handle_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('r') => self.selected = true,
            KeyCode::Esc => {
                if self.selected {
                    self.reset();
                }
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.selected = false;
    }
}
