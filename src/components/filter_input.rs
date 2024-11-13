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
enum InputMode {
    Editing,
    #[default]
    Normal,
}

#[derive(Default)]
pub struct FilterInput {
    active: bool,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    input: String,
    input_mode: InputMode,
    title: String,
    character_index: usize,
}

impl FilterInput {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Self::default()
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
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
                self.input_mode = InputMode::Editing;
                self.active = true;
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::TransmitSubmittedText(self.input.to_string()))?;
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::EnterInsertMode)?;
            }
            Action::SelectTableMode
            | Action::SelectDataMode
            | Action::SelectingRegion
            | Action::ViewTableDataRowDetail => self.active = false,
            Action::NewCharacter(c) => {
                if self.active {
                    // self.input.push(c);
                    self.enter_char(c);
                    self.command_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::TransmitSubmittedText(self.input.to_string()))?;
                }
            }
            Action::DeleteCharacter => {
                if self.active {
                    // self.input.pop();
                    self.delete_char();
                    self.command_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::TransmitSubmittedText(self.input.to_string()))?;
                }
            }
            Action::ExitInsertMode => {
                self.input_mode = InputMode::Normal;
                self.active = false;
                self.input.clear();
                self.reset_cursor();
            }
            Action::SubmitText => {
                self.input_mode = InputMode::Normal;
                let command_tx_lock = self.command_tx.as_ref().unwrap();

                self.active = false;

                command_tx_lock.send(Action::ExitInsertMode)?;
                command_tx_lock.send(Action::SelectTableMode)?;
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

        match self.input_mode {
            InputMode::Editing => {
                frame.set_cursor_position(Position::new(
                    bottom_left.x + self.character_index as u16 + 1,
                    bottom_left.y + 1,
                ));
            }
            InputMode::Normal => {}
        }

        Ok(())
    }
}
