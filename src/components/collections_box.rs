use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::Widget;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::Component;

pub struct CollectionsBox {
    pub selected: bool,
}

impl CollectionsBox {
    pub fn new() -> Self {
        Self { selected: false }
    }
}

impl Component for CollectionsBox {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Collections");

        if self.selected {
            block = block.border_style(Style::default().fg(Color::Green));
        }

        Paragraph::new("").block(block).render(area, buf);
    }

    fn handle_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('c') => self.selected = true,
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.selected = false;
    }
}
