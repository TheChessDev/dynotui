use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    prelude::Widget,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::message::Message;

use super::{MutableComponent, ACTIVE_PANE_COLOR};

pub struct FilterInput {
    pub active: bool,
    pub input: String,
    pub title: String,
}

impl FilterInput {
    pub fn new(title: &str) -> Self {
        Self {
            active: false,
            input: String::new(),
            title: title.to_string(),
        }
    }
}

impl MutableComponent for FilterInput {
    fn render(&mut self, area: Rect, buf: &mut Buffer, active: bool) {
        self.active = active;

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(self.title.clone())
            .border_style(if self.active {
                Style::default().fg(ACTIVE_PANE_COLOR)
            } else {
                Style::default().fg(Color::Gray)
            });

        let paragraph = Paragraph::new(self.input.clone()).block(block);
        paragraph.render(area, buf);
    }

    fn handle_event<F>(&mut self, event: KeyEvent, send_message: F)
    where
        F: FnOnce(Message),
    {
        match event.code {
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Enter => send_message(Message::ApplyCollectionsFilter),
            KeyCode::Esc => {
                self.reset();
                send_message(Message::CancelFilteringCollectionMode)
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.input.clear();
    }
}
