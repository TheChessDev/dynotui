use color_eyre::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{EMERALD, VIOLET};

use clipboard::{ClipboardContext, ClipboardProvider};
use ratatui::style::Color;
use ratatui::widgets::{
    Clear, List, ListItem, ListState, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, StatefulWidget,
};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
};
use serde_json::Value;
use style::palette::material::INDIGO;
use symbols::scrollbar;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::config::Config;

use super::Component;

#[derive(Default)]
pub struct DataBox {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    active: bool,
    title: String,
    records: Vec<String>,
    filtered_records: Vec<String>,
    has_more: bool,
    list_state: ListState,
    selected_row: String,
    collection_name: String,
    fetching: bool,
    aprox_count: i64,
    scroll_bar_state: ScrollbarState,
    mode: Mode,
    filter_input: String,
    character_index: usize,
}

#[derive(Default)]
enum Mode {
    #[default]
    View,
    Filtering,
    Querying,
}

impl DataBox {
    pub fn new() -> Self {
        Self {
            title: "Data".to_string(),
            ..Self::default()
        }
    }

    pub fn apply_filter(&mut self) {
        if self.filter_input.is_empty() {
            // If no filter input, show all records
            self.filtered_records = self.records.clone();
        } else {
            let matcher = SkimMatcherV2::default();
            let keywords: Vec<&str> = self.filter_input.split_whitespace().collect();

            self.filtered_records = self
                .records
                .iter()
                .filter(|row| {
                    // Parse each record as JSON
                    if let Ok(parsed_row) = serde_json::from_str::<Value>(row) {
                        // Check if all keywords are found in the JSON object
                        keywords.iter().all(|keyword| {
                            self.keyword_matches_json(keyword, &parsed_row, &matcher)
                        })
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
        }
    }

    // Helper function to check if a keyword matches any field or value in the JSON
    fn keyword_matches_json(&self, keyword: &str, json: &Value, matcher: &SkimMatcherV2) -> bool {
        match json {
            Value::Object(map) => {
                for (key, value) in map {
                    // Check if the keyword matches the field name
                    if matcher.fuzzy_match(key, keyword).is_some() {
                        return true;
                    }
                    // Recursively check values in case of nested objects/arrays
                    if self.keyword_matches_json(keyword, value, matcher) {
                        return true;
                    }
                }
                false
            }
            Value::Array(arr) => {
                // Check each item in the array
                arr.iter()
                    .any(|value| self.keyword_matches_json(keyword, value, matcher))
            }
            Value::String(s) => matcher.fuzzy_match(s, keyword).is_some(),
            Value::Number(n) => matcher.fuzzy_match(&n.to_string(), keyword).is_some(),
            Value::Bool(b) => matcher.fuzzy_match(&b.to_string(), keyword).is_some(),
            Value::Null => false,
        }
    }

    pub fn set_title(&mut self, new_title: &str) {
        self.title = new_title.to_string();
    }

    fn select_next(&mut self) {
        self.list_state.select_next();
        self.update_scroll_pos(self.list_state.selected().unwrap());
    }

    fn select_previous(&mut self) {
        self.list_state.select_previous();
        self.update_scroll_pos(self.list_state.selected().unwrap());
    }

    pub fn select_first(&mut self) {
        self.list_state.select_first();
        self.update_scroll_pos(self.list_state.selected().unwrap());
    }

    fn select_last(&mut self) {
        self.list_state.select_last();
        self.update_scroll_pos(self.list_state.selected().unwrap());
    }

    fn scroll_up(&mut self) {
        self.list_state.scroll_up_by(5);
        self.update_scroll_pos(self.list_state.selected().unwrap());
    }

    fn scroll_down(&mut self) {
        self.list_state.scroll_down_by(5);
        self.update_scroll_pos(self.list_state.selected().unwrap());
    }

    fn set_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.selected_row = self.records[i].to_string();
        }
    }

    fn update_scroll_pos(&mut self, pos: usize) {
        self.scroll_bar_state = self.scroll_bar_state.position(pos);
    }

    fn copy_selected_row_to_clipboard(&self) {
        if let Some(i) = self.list_state.selected() {
            let selected_row = &self.records[i];

            let mut ctx: ClipboardContext =
                ClipboardProvider::new().expect("Failed to access clipboard");
            ctx.set_contents(selected_row.clone())
                .expect("Failed to copy to clipboard");
        }
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.filter_input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.filter_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.filter_input.len())
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.filter_input.chars().count())
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
            let before_char_to_delete = self.filter_input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.filter_input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.filter_input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }
}

impl Component for DataBox {
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
            Action::SelectDataMode => self.active = true,
            Action::SelectingRegion
            | Action::FilteringTables
            | Action::SelectTableMode
            | Action::ViewTableDataRowDetail => self.active = false,
            Action::TransmitSelectedTable(table) => {
                self.set_title(&table);
                self.collection_name = table.clone();
            }
            Action::TransmitTableData(data, has_more) => {
                self.records = data;
                self.has_more = has_more;
                self.list_state.select_first();
                self.apply_filter();
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::StopLoading)?;
            }
            Action::SelectTableDataRowPrev => {
                self.select_previous();
            }
            Action::SelectTableDataRowNext => {
                self.select_next();
                if let Some(selected) = self.list_state.selected() {
                    if selected >= self.records.len() - 5 && self.has_more && !self.fetching {
                        self.fetching = true;
                        let command_ref = self.command_tx.as_ref().unwrap();
                        command_ref
                            .send(Action::StartLoading("Loading More Table Data".to_string()))?;
                        command_ref
                            .send(Action::FetchMoreTableData(self.collection_name.clone()))?;
                    }
                }
            }
            Action::SelectTableDataRowScrollUp => {
                self.scroll_up();
            }
            Action::SelectTableDataRowScrollDown => {
                self.scroll_down();
                if let Some(selected) = self.list_state.selected() {
                    if selected >= self.records.len() - 5 && self.has_more && !self.fetching {
                        self.fetching = true;
                        let command_ref = self.command_tx.as_ref().unwrap();
                        command_ref
                            .send(Action::StartLoading("Loading More Table Data".to_string()))?;
                        command_ref
                            .send(Action::FetchMoreTableData(self.collection_name.clone()))?;
                    }
                }
            }
            Action::SelectTableDataRowFirst => {
                self.select_first();
            }
            Action::SelectTableDataRowLast => {
                self.select_last();
                if self.has_more && !self.fetching {
                    self.fetching = true;
                    let command_ref = self.command_tx.as_ref().unwrap();
                    command_ref
                        .send(Action::StartLoading("Loading More Table Data".to_string()))?;
                    command_ref.send(Action::FetchMoreTableData(self.collection_name.clone()))?;
                }
            }
            Action::SelectTableDataRow => {
                self.set_selected();

                if !self.selected_row.is_empty() {
                    let command_tx = self.command_tx.as_ref().unwrap();

                    command_tx.send(Action::ViewTableDataRowDetail)?;
                    command_tx.send(Action::TransmitSelectedTableDataRow(
                        self.selected_row.clone(),
                    ))?;
                }
            }
            Action::TransmitNextBatcTableData(data, has_more) => {
                self.fetching = false;
                self.has_more = has_more;
                self.records.extend(data);
                self.apply_filter();

                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::StopLoading)?;
            }
            Action::FetchTableData(_) => {
                self.records.clear();
            }
            Action::ApproximateTableDataCount(count) => {
                self.aprox_count = count;
            }
            Action::SelectTableDataRowCopyToClipboard => {
                self.copy_selected_row_to_clipboard();
            }
            Action::FilterTableData => self.mode = Mode::Filtering,
            Action::ExitFilterTableData => {
                self.mode = Mode::View;
                self.filter_input.clear();
                self.character_index = 0;
                self.apply_filter();
            }
            Action::ExitQueryTableData => self.mode = Mode::View,
            Action::NewFilterDataCharacter(c) => {
                if self.active {
                    self.enter_char(c);
                    self.select_first();
                    self.apply_filter();
                }
            }
            Action::DeleteFilterDataCharacter => {
                if self.active {
                    self.delete_char();
                    self.apply_filter();
                }
            }
            Action::SubmitFilterDataText => {
                self.mode = Mode::View;
            }
            Action::ClearTableDataFilter => {
                self.filter_input.clear();
                self.character_index = 0;
                self.apply_filter();
            }
            Action::QueryTableData => self.mode = Mode::Querying,
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [top, bottom] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        let [_, right] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(top);

        let [_, bottom_right] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(bottom);

        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(self.title.clone());

        if self.active {
            block = block.border_style(Style::default().fg(EMERALD.c300));
        }

        let items: Vec<ListItem> = self
            .filtered_records
            .iter()
            .map(|record| ListItem::new(record.clone()))
            .collect();

        self.scroll_bar_state = self.scroll_bar_state.content_length(items.len());

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::new().bg(VIOLET.c600).add_modifier(Modifier::BOLD));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(None)
            .track_symbol(None)
            .end_symbol(None);

        StatefulWidget::render(list, right, frame.buffer_mut(), &mut self.list_state);

        StatefulWidget::render(
            scrollbar,
            right.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            frame.buffer_mut(),
            &mut self.scroll_bar_state,
        );

        match self.mode {
            Mode::View => {
                let view_mode = if self.filter_input.is_empty() {
                    "Fetched"
                } else {
                    "Viewing"
                };
                let status_text = format!(
                    "{} {} Items (Scanned: {})",
                    view_mode,
                    self.filtered_records.len(),
                    self.aprox_count
                );

                Paragraph::new(status_text)
                    .block(Block::default().padding(Padding::horizontal(2)))
                    .style(Style::new().fg(INDIGO.c700))
                    .render(bottom_right, frame.buffer_mut());
            }
            Mode::Querying => {
                let [_, y_middle, _] = Layout::vertical([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .areas(area);

                let [_, middle, _] = Layout::horizontal([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .areas(y_middle);

                let [top, bottom] =
                    Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .areas(middle);

                let top_block = Block::new()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(EMERALD.c300))
                    .style(Style::new().bg(Color::Black))
                    .title("Query Table");

                let bottom_block = Block::new()
                    .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(EMERALD.c300))
                    .style(Style::new().bg(Color::Black));

                frame.render_widget(Clear, middle);

                Paragraph::new("Top")
                    .block(top_block)
                    .style(Style::new().bg(Color::Black))
                    .render(top, frame.buffer_mut());
                Paragraph::new("Bottom")
                    .block(bottom_block)
                    .style(Style::new().bg(Color::Black))
                    .render(bottom, frame.buffer_mut());
            }
            Mode::Filtering => {
                let [search_left, search_right] =
                    Layout::horizontal([Constraint::Length(8), Constraint::Min(0)])
                        .areas(bottom_right);

                let paragraph = Paragraph::new(self.filter_input.clone());
                paragraph.render(search_right, frame.buffer_mut());

                frame.set_cursor_position(Position::new(
                    search_right.x + self.character_index as u16,
                    search_right.y,
                ));

                Paragraph::new("Search:")
                    .style(Style::new().fg(INDIGO.c700))
                    .render(search_left, frame.buffer_mut());
            }
        }

        Ok(())
    }
}
