use color_eyre::Result;

use ratatui::layout::{Constraint, Layout};
use ratatui::style::Color;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};
use ratatui::Frame;
use ratatui::{layout::Rect, style::Style};
use throbber_widgets_tui::ThrobberState;

use crate::action::Action;

use super::Component;

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

    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }
}

impl Component for LoadingBox {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => self.on_tick(),
            Action::StartLoading(message) => {
                self.active = true;
                self.set_message(&message);
            }
            Action::StopLoading => self.active = false,
            _ => {}
        };

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [left, _] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(area);
        let [_, bottom] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(left);

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

            StatefulWidget::render(full, bottom, frame.buffer_mut(), &mut self.loading_state);
        } else {
            Paragraph::new("").render(bottom, frame.buffer_mut());
        }

        Ok(())
    }
}
