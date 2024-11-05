use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};
use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::widgets::{List, ListItem, ListState, StatefulWidget};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use serde_json::{Map, Value};

use crate::app::Message;
use crate::util::dynamodb_to_json;

use super::{MutableComponent, ACTIVE_PANE_COLOR, ROW_HOVER_COLOR};

pub struct DataBox {
    pub selected: bool,
    pub title: String,
    pub content: String,
    pub records: Vec<String>,
    pub has_more: bool,
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
    pub list_state: ListState,
}

impl DataBox {
    pub fn new() -> Self {
        Self {
            selected: false,
            title: "Data".to_string(),
            content: String::new(),
            records: Vec::new(),
            has_more: false,
            last_evaluated_key: None,
            list_state: ListState::default(),
        }
    }

    pub fn set_title(&mut self, new_title: &str) {
        self.title = new_title.to_string();
    }

    pub fn set_content(&mut self, new_content: &str) {
        self.content = new_content.to_string();
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

    fn handle_event<F>(&mut self, event: KeyEvent, _send_message: F)
    where
        F: FnOnce(Message),
    {
        match event.code {
            KeyCode::Esc => {
                self.reset();
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.selected = false;
    }
}
