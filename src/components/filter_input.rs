use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    prelude::Widget,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::{Component, SELECTED_COLOR};

pub struct FilterInput {
    pub active: bool,
    pub input: String,
}

impl FilterInput {
    pub fn new() -> Self {
        Self {
            active: false,
            input: String::new(),
        }
    }
}

impl Component for FilterInput {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Filter Collections")
            .border_style(if self.active {
                Style::default().fg(SELECTED_COLOR)
            } else {
                Style::default().fg(Color::Gray)
            });

        let paragraph = Paragraph::new(self.input.clone()).block(block);
        paragraph.render(area, buf);
    }

    fn handle_event(&mut self, event: KeyEvent) {
        if self.active {
            match event.code {
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Enter | KeyCode::Esc => self.active = false,
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        self.input.clear();
    }
}
