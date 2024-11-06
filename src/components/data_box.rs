use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use ratatui::widgets::{List, ListItem, ListState, StatefulWidget};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
};
use serde_json::{Map, Value};

use crate::app::Message;
use crate::util::dynamodb_to_json;

use super::{MutableComponent, ACTIVE_PANE_COLOR, ROW_HOVER_COLOR};

pub struct DataBox {
    pub selected: bool,
    pub title: String,
    pub records: Vec<String>,
    pub has_more: bool,
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
    pub list_state: ListState,
    pub selected_row: String,
}

impl DataBox {
    pub fn new() -> Self {
        Self {
            selected: false,
            title: "Data".to_string(),
            records: Vec::new(),
            has_more: true,
            last_evaluated_key: None,
            list_state: ListState::default(),
            selected_row: String::new(),
        }
    }

    pub fn set_title(&mut self, new_title: &str) {
        self.title = new_title.to_string();
    }

    pub async fn load_data(&mut self, client: &Client, collection_name: &str) -> Result<(), Error> {
        if !self.has_more {
            return Ok(()); // No more records to load
        }

        let mut request = client.scan().table_name(collection_name).limit(100);

        // Add `ExclusiveStartKey` if continuing from a previous batch
        if let Some(ref key) = self.last_evaluated_key {
            for (k, v) in key.iter() {
                request = request.exclusive_start_key(k.clone(), v.clone());
            }
        }

        let response = request.send().await?;

        if let Some(items) = response.items {
            self.records.extend(items.into_iter().map(|item| {
                let mut json_item = Map::new();
                for (k, v) in item {
                    json_item.insert(k, dynamodb_to_json(v));
                }
                Value::Object(json_item).to_string()
            }));
        }

        // Update pagination state
        self.last_evaluated_key = response.last_evaluated_key.map(|key| {
            key.into_iter()
                .map(|(k, v)| (k, v))
                .collect::<HashMap<String, AttributeValue>>()
        });

        self.has_more = self.last_evaluated_key.is_some();

        Ok(())
    }

    fn select_none(&mut self) {
        self.list_state.select(None);
    }

    fn select_next(&mut self) {
        self.list_state.select_next();
    }

    fn select_previous(&mut self) {
        self.list_state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.list_state.select_first();
    }

    fn select_last(&mut self) {
        self.list_state.select_last();
    }

    fn scroll_up(&mut self) {
        self.list_state.scroll_up_by(5);
    }

    fn scroll_down(&mut self) {
        self.list_state.scroll_down_by(5);
    }

    fn set_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.selected_row = self.records[i].to_string();
        }
    }
}

impl MutableComponent for DataBox {
    fn render(&mut self, area: Rect, buf: &mut Buffer, active: bool) {
        self.selected = active;

        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(self.title.clone());

        if self.selected {
            block = block.border_style(Style::default().fg(ACTIVE_PANE_COLOR));
        }

        let items: Vec<ListItem> = self
            .records
            .iter()
            .map(|record| ListItem::new(record.clone()))
            .collect();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(ROW_HOVER_COLOR));

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }

    fn handle_event<F>(&mut self, event: KeyEvent, send_message: F)
    where
        F: FnOnce(Message),
    {
        match event.code {
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                if let Some(selected) = self.list_state.selected() {
                    if selected >= self.records.len() - 5 && self.has_more {
                        send_message(Message::LoadMoreData);
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => {
                self.select_last();
                if self.has_more {
                    send_message(Message::LoadMoreData);
                }
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.set_selected();
            }

            KeyCode::Esc => {
                self.reset();
            }
            _ => {}
        }

        if event == KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL) {
            self.scroll_down();
        }

        if event == KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL) {
            self.scroll_up();
        }
    }

    fn reset(&mut self) {
        self.records.clear();
        self.has_more = true;
        self.last_evaluated_key = None;
        self.title = "Data".to_string();
        self.selected_row = String::new();
        self.list_state = ListState::default();

        self.selected = false;
    }
}
