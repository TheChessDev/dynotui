use std::sync::Arc;

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;
use ratatui::widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

use tokio::runtime::Runtime;

use super::MutableComponent;

const SELECTED_STYLE: Style = Style::new()
    .bg(Color::LightMagenta)
    .add_modifier(Modifier::BOLD);

pub struct CollectionsBox {
    pub selected: bool,
    pub collections: Vec<String>,
    pub collections_list: CollectionsList,
    pub selected_collection: String,
}

pub struct CollectionsList {
    state: ListState,
}

impl CollectionsBox {
    pub fn new() -> Self {
        let collections_list = CollectionsList {
            state: ListState::default(),
        };

        Self {
            selected: false,
            collections: Vec::new(),
            collections_list,
            selected_collection: String::new(),
        }
    }

    pub fn load_collections(&mut self, _region: &str) {
        let rt = Runtime::new().unwrap();

        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");

        let config = Arc::new(rt.block_on(async {
            aws_config::defaults(BehaviorVersion::v2024_03_28())
                .region(region_provider)
                .load()
                .await
        }));

        let client = Client::new(&config);

        match rt.block_on(async { client.list_tables().send().await }) {
            Ok(output) => {
                self.collections = output
                    .table_names()
                    .iter()
                    .map(|name| name.clone())
                    .collect();
            }
            Err(_) => self.collections = vec!["Error loading collections.".to_string()],
        }
    }

    fn select_none(&mut self) {
        self.collections_list.state.select(None);
    }

    fn select_next(&mut self) {
        self.collections_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.collections_list.state.select_previous();
    }

    fn select_first(&mut self) {
        self.collections_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.collections_list.state.select_last();
    }

    fn set_selected(&mut self) {
        if let Some(i) = self.collections_list.state.selected() {
            self.selected_collection = i.to_string();
        }
    }
}

impl MutableComponent for CollectionsBox {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Collections");

        if self.selected {
            block = block.border_style(Style::default().fg(Color::Green));
        }

        let items: Vec<ListItem> = self
            .collections
            .iter()
            .map(|name| ListItem::new(name.clone()))
            .collect();

        let collection_list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(collection_list, area, buf, &mut self.collections_list.state);
    }

    fn handle_event(&mut self, event: KeyEvent) {
        if !self.selected {
            match event.code {
                KeyCode::Char('c') => self.selected = true,
                _ => {}
            }
        } else {
            match event.code {
                KeyCode::Char('c') => self.selected = true,
                KeyCode::Char('h') | KeyCode::Left => self.select_none(),
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                KeyCode::Char('G') | KeyCode::End => self.select_last(),
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                    self.set_selected();
                }
                KeyCode::Esc => {
                    self.reset();
                }
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        self.selected = false;
    }
}
