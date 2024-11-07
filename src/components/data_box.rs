use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{EMERALD, VIOLET};
use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};
use ratatui::style::Color;
use ratatui::widgets::{List, ListItem, ListState, StatefulWidget};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
};
use serde_json::{Map, Value};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::config::Config;
use crate::util::dynamodb_to_json;

use super::Component;

#[derive(Default)]
pub struct DataBox {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    pub active: bool,
    pub title: String,
    pub records: Vec<String>,
    pub has_more: bool,
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
    pub list_state: ListState,
    pub selected_row: String,
}

impl DataBox {
    pub fn new() -> Self {
        Self::default()
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
        self.last_evaluated_key = response
            .last_evaluated_key
            .map(|key| key.into_iter().collect::<HashMap<String, AttributeValue>>());

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

    fn reset(&mut self) {
        self.records.clear();
        self.has_more = true;
        self.last_evaluated_key = None;
        self.title = "Data".to_string();
        self.selected_row = String::new();
        self.list_state = ListState::default();

        self.active = false;
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
            Action::SelectingData => self.active = true,
            Action::SelectingRegion | Action::FilteringTables | Action::SelectingTable => {
                self.active = false
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [top, _] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        let [_, right] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(top);

        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(self.title.clone());

        if self.active {
            block = block.border_style(Style::default().fg(EMERALD.c300));
        }

        let items: Vec<ListItem> = self
            .records
            .iter()
            .map(|record| ListItem::new(record.clone()))
            .collect();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::new().bg(VIOLET.c600).add_modifier(Modifier::BOLD));

        StatefulWidget::render(list, right, frame.buffer_mut(), &mut self.list_state);

        Ok(())
    }
}
