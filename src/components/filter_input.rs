use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::{
    layout::Rect,
    prelude::Widget,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use style::palette::tailwind::EMERALD;
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, config::Config};

use super::Component;

#[derive(Default)]
pub struct FilterInput {
    active: bool,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    input: String,
    title: String,
}

impl FilterInput {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Self::default()
        }
    }
}

impl Component for FilterInput {
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
            Action::FilteringTables => {
                self.active = true;
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::EnterInsertMode)?;
            }
            Action::SelectingTable | Action::SelectingData | Action::SelectingRegion => {
                self.active = false
            }
            Action::NewCharacter(c) => {
                if self.active {
                    self.input.push(c);
                    self.command_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::TransmitSubmittedText(self.input.to_string()))?;
                }
            }
            Action::DeleteCharacter => {
                if self.active {
                    self.input.pop();
                }
            }
            Action::ExitInsertMode => {
                self.active = false;
                self.input.clear();
            }
            Action::SubmitText => {
                let command_tx_lock = self.command_tx.as_ref().unwrap();

                self.active = false;

                command_tx_lock.send(Action::ExitInsertMode)?;
                command_tx_lock.send(Action::SelectingTable)?;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [top, _] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        let [left, _] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(top);

        let [_, _, bottom_left] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(left);
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(self.title.clone())
            .border_style(if self.active {
                Style::default().fg(EMERALD.c300)
            } else {
                Style::default().fg(Color::Gray)
            });

        let paragraph = Paragraph::new(self.input.clone()).block(block);
        paragraph.render(bottom_left, frame.buffer_mut());

        Ok(())
    }
}
