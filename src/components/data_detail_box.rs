use clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{palette::tailwind::EMERALD, Color, Style},
    symbols::scrollbar,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};
use serde_json::{Error, Value};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, config::Config};

use super::Component;

#[derive(Default)]
pub struct DataDetailBox {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    active: bool,
    title: String,
    row: String,
    vertical_scroll: usize,
    horizontal_scroll: usize,
    vertical_scroll_state: ScrollbarState,
    horizontal_scroll_state: ScrollbarState,
}

impl DataDetailBox {
    pub fn new() -> Self {
        Self::default()
    }

    fn pretty_print_row(&self) -> Result<Vec<Line>, Error> {
        let parsed_json: Value = serde_json::from_str(&self.row)?;

        let lines = self.json_to_lines(&parsed_json, 0);

        Ok(lines)
    }

    fn json_to_lines(&self, json: &Value, indent: usize) -> Vec<Line> {
        let mut lines = vec![];

        match json {
            Value::Object(map) => {
                // Opening brace for the object with proper indentation
                lines.push(Line::from(Span::styled(
                    " ".repeat(indent) + "{",
                    Style::default().fg(Color::Cyan),
                )));

                for (key, value) in map.iter() {
                    // Key with indentation
                    let mut line = vec![
                        Span::raw(" ".repeat(indent + 2)),
                        Span::styled(format!(r#""{}""#, key), Style::default().fg(Color::Yellow)),
                        Span::raw(": "),
                    ];

                    // Add value spans with appropriate indentation for each line
                    if value.is_object() || value.is_array() {
                        // For nested objects/arrays, start a new line for each level
                        lines.push(Line::from(line)); // Push the key line
                        let nested_lines = self.json_to_lines(value, indent + 2);
                        for nested_line in nested_lines {
                            lines.push(nested_line); // Add each nested line directly
                        }
                    } else {
                        line.extend(self.json_value_to_spans(value, indent + 2));
                        lines.push(Line::from(line));
                    }
                }

                // Closing brace for the object with indentation
                lines.push(Line::from(Span::styled(
                    " ".repeat(indent) + "}",
                    Style::default().fg(Color::Cyan),
                )));
            }
            Value::Array(array) => {
                // Opening bracket for the array with proper indentation
                lines.push(Line::from(Span::styled(
                    " ".repeat(indent) + "[",
                    Style::default().fg(Color::Cyan),
                )));

                for value in array {
                    // Add each array item with indentation
                    if value.is_object() || value.is_array() {
                        let nested_lines = self.json_to_lines(value, indent + 2);
                        for nested_line in nested_lines {
                            lines.push(Line::from(vec![
                                Span::raw(" ".repeat(indent + 2)),
                                Span::raw(""),
                            ]));
                            lines.push(nested_line);
                        }
                    } else {
                        lines.push(Line::from(vec![
                            Span::raw(" ".repeat(indent + 2)),
                            self.json_value_to_spans(value, indent + 2)[0].clone(),
                        ]));
                    }
                }

                // Closing bracket for the array with indentation
                lines.push(Line::from(Span::styled(
                    " ".repeat(indent) + "]",
                    Style::default().fg(Color::Cyan),
                )));
            }
            _ => lines.push(Line::from(Span::raw(json.to_string()))),
        }

        lines
    }

    fn json_value_to_spans(&self, value: &Value, _indent: usize) -> Vec<Span> {
        match value {
            Value::String(s) => vec![Span::styled(
                format!(r#""{}""#, s),
                Style::default().fg(Color::Green),
            )],
            Value::Number(n) => vec![Span::styled(
                n.to_string(),
                Style::default().fg(Color::Magenta),
            )],
            Value::Bool(b) => vec![Span::styled(b.to_string(), Style::default().fg(Color::Red))],
            Value::Null => vec![Span::styled("null", Style::default().fg(Color::Gray))],
            Value::Object(_) | Value::Array(_) => {
                // For nested objects/arrays, we defer to `json_to_lines` to handle them properly.
                vec![]
            }
        }
    }

    fn copy_selected_row_to_clipboard(&self) {
        let mut ctx: ClipboardContext =
            ClipboardProvider::new().expect("Failed to access clipboard");

        ctx.set_contents(self.row.clone())
            .expect("Failed to copy to clipboard");
    }
}

impl Component for DataDetailBox {
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
            Action::ViewTableDataRowDetail => {
                self.active = true;
                self.horizontal_scroll = 0;
                self.vertical_scroll = 0;
            }
            Action::SelectingRegion
            | Action::FilteringTables
            | Action::SelectTableMode
            | Action::SelectDataMode => self.active = false,
            Action::TransmitSelectedTableDataRow(row) => self.row = row,
            Action::ExitViewTableDataRowMode => {
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::SelectDataMode)?;
            }
            Action::ViewTableDataRowScrollPrev => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            Action::ViewTableDataRowScrollNext => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            Action::ViewTableDataRowScrollUp => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(10);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            Action::ViewTableDataRowScrollDown => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(10);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            Action::ViewTableDataRowScrollLeft => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                self.horizontal_scroll_state = self
                    .horizontal_scroll_state
                    .position(self.horizontal_scroll);
            }
            Action::ViewTableDataRowScrollRight => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
                self.horizontal_scroll_state = self
                    .horizontal_scroll_state
                    .position(self.horizontal_scroll);
            }
            Action::ViewTableDataRowCopyToClipboard => {
                self.copy_selected_row_to_clipboard();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        let [_, y_middle, _] = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .areas(area);

        let [_, middle, _] = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .areas(y_middle);

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(EMERALD.c300))
            .style(Style::new().bg(Color::Black))
            .title(self.title.clone());

        let lines: Vec<Line> = self.pretty_print_row().unwrap_or_default();

        let longest_line_width = lines.iter().map(|line| line.width()).max().unwrap_or(0);
        let total_lines = lines.len();

        let p = Paragraph::new(lines)
            .block(block)
            .style(Style::new().bg(Color::Black))
            .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16));

        frame.render_widget(Clear, middle);
        frame.render_widget(p, middle);

        self.vertical_scroll_state = self.vertical_scroll_state.content_length(total_lines);

        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .content_length(longest_line_width);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
            middle.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.vertical_scroll_state,
        );

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .symbols(scrollbar::HORIZONTAL)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
            middle.inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
            &mut self.horizontal_scroll_state,
        );

        Ok(())
    }
}
