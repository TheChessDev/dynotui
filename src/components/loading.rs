use ratatui::style::Color;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};
use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect, style::Style};
use throbber_widgets_tui::ThrobberState;

use crate::message::Message;

use super::MutableComponent;

pub struct LoadingBox {
    pub active: bool,
    pub loading_state: ThrobberState,
    pub message: String,
}

impl LoadingBox {
    pub fn new() -> Self {
        Self {
            active: false,
            loading_state: ThrobberState::default(),
            message: "Loading...".to_string(),
        }
    }

    pub fn on_tick(&mut self) {
        self.loading_state.calc_next();
    }

    pub fn start_loading(&mut self) {
        self.active = true;
    }

    pub fn end_loading(&mut self) {
        self.active = false;
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }
}

impl MutableComponent for LoadingBox {
    fn render(&mut self, area: Rect, buf: &mut Buffer, _active: bool) {
        if self.active {
            let full = throbber_widgets_tui::Throbber::default()
                .label(self.message.clone())
                .style(Style::default().fg(ratatui::style::Color::Cyan))
                .throbber_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                )
                .throbber_set(throbber_widgets_tui::ASCII)
                .use_type(throbber_widgets_tui::WhichUse::Spin);

            StatefulWidget::render(full, area, buf, &mut self.loading_state);
        } else {
            Paragraph::new("not loading").render(area, buf);
        }
    }

    fn handle_event<F>(&mut self, _event: KeyEvent, _send_message: F)
    where
        F: FnOnce(Message),
    {
    }

    fn reset(&mut self) {
        self.active = false;
    }
}
