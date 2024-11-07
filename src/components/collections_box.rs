use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::style::Color;
use ratatui::widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::config::Config;
use crate::constants::{ACTIVE_PANE_COLOR, LIST_ITEM_SELECTED_STYLE};

use super::Component;

#[derive(Debug, Default)]
pub struct CollectionsBox {
    active: bool,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    collections: Vec<String>,
    filtered_collections: Vec<String>,
    list_state: ListState,
    selected_collection: String,
    filter_text: String,
    region: String,
}

impl CollectionsBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_collections = self.collections.clone();
        } else {
            let matcher = SkimMatcherV2::default();
            self.filtered_collections = self
                .collections
                .iter()
                .filter(|collection| matcher.fuzzy_match(collection, &self.filter_text).is_some())
                .cloned()
                .collect();
        }
    }

    pub async fn get_client(&self) -> Client {
        let region = "us-east-1";
        let region_provider = RegionProviderChain::default_provider().or_else(region);
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(region_provider)
            .load()
            .await;

        Client::new(&config)
    }

    pub async fn load_collections(&mut self) {
        let client = self.get_client().await;

        let mut last_evaluated_table_name = None;
        self.collections.clear();

        loop {
            let request = client
                .list_tables()
                .set_exclusive_start_table_name(last_evaluated_table_name.clone());

            match request.send().await {
                Ok(output) => {
                    let table_names = output.table_names();

                    for name in table_names {
                        self.collections.push(name.clone());
                    }

                    last_evaluated_table_name =
                        output.last_evaluated_table_name().map(|s| s.to_string());

                    if last_evaluated_table_name.is_none() {
                        break;
                    }
                }
                Err(_) => {
                    self.collections = vec!["Error loading collections.".to_string()];
                    break;
                }
            }
        }

        self.apply_filter();
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

    fn set_selected(&mut self) -> bool {
        if let None = self.list_state.selected() {
            return false;
        }

        let col_indx = self.list_state.selected().unwrap();
        let new_col_name = self.filtered_collections[col_indx].to_string();

        if new_col_name == self.selected_collection {
            return false;
        }

        self.selected_collection = new_col_name;

        true
    }

    fn reset(&mut self) {
        self.active = false;
    }
}

impl Component for CollectionsBox {
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
            Action::SelectingTable => {
                self.active = true;
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::FetchTables)?;
            }
            Action::FilteringTables | Action::SelectingRegion | Action::SelectingData => {
                self.active = false
            }
            Action::TransmitSubmittedText(text) => {
                self.filter_text = text.clone();
                self.apply_filter();
            }
            Action::TransmitTables(tables) => {
                self.collections = tables;
                self.apply_filter();
            }
            _ => {}
        }
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let [top, _] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        let [left, _] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(top);

        let [_, middle_left, _] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(left);

        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Tables");

        if self.active {
            block = block.border_style(Style::default().fg(ACTIVE_PANE_COLOR));
        }

        let items: Vec<ListItem> = self
            .filtered_collections
            .iter()
            .map(|name| ListItem::new(name.clone()))
            .collect();

        let collection_list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_style(LIST_ITEM_SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(
            collection_list,
            middle_left,
            frame.buffer_mut(),
            &mut self.list_state,
        );

        Ok(())
    }
}
