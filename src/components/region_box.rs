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

use crate::app::Message;

use super::{MutableComponent, ACTIVE_PANE_COLOR};

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

impl MutableComponent for AWSRegionBox {
    fn render(&mut self, area: Rect, buf: &mut Buffer, active: bool) {
        self.selected = active;
        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("AWS Region");

        if self.selected {
            block = block.border_style(Style::default().fg(ACTIVE_PANE_COLOR));
        }

        Paragraph::new(self.region.clone())
            .alignment(Alignment::Center)
            .block(block)
            .render(area, buf);
    }

    fn handle_event<F>(&mut self, event: KeyEvent, _send_message: F)
    where
        F: FnOnce(Message),
    {
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
