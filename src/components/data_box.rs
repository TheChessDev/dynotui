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

pub struct DataBox {
    pub selected: bool,
}

impl DataBox {
    pub fn new() -> Self {
        Self { selected: false }
    }
}

impl Component for DataBox {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Data");

        if self.selected {
            block = block.border_style(Style::default().fg(Color::Green));
        }

        Paragraph::new("").block(block).render(area, buf);
    }

    fn handle_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('d') => self.selected = true,
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
