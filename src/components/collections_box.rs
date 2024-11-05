use std::sync::Arc;

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier};
use ratatui::widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::{palette::tailwind::VIOLET, Style},
    widgets::{Block, BorderType, Borders},
};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use tokio::runtime::Runtime;

use super::filter_input::FilterInput;
use super::{Component, MutableComponent, SELECTED_COLOR};

const SELECTED_STYLE: Style = Style::new().bg(VIOLET.c600).add_modifier(Modifier::BOLD);

pub struct CollectionsBox {
    pub selected: bool,
    pub collections: Vec<String>,
    pub filtered_collections: Vec<String>,
    pub collections_list: CollectionsList,
    pub selected_collection: String,
    pub filter_input: FilterInput,
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
            filtered_collections: Vec::new(),
            collections_list,
            selected_collection: String::new(),
            filter_input: FilterInput::new(),
        }
    }

    pub fn apply_filter(&mut self) {
        if self.filter_input.input.is_empty() {
            self.filtered_collections = self.collections.clone();
        } else {
            let matcher = SkimMatcherV2::default();
            self.filtered_collections = self
                .collections
                .iter()
                .filter(|collection| {
                    matcher
                        .fuzzy_match(collection, &self.filter_input.input)
                        .is_some()
                })
                .cloned()
                .collect();
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
        let mut last_evaluated_table_name = None;
        self.collections.clear();

        loop {
            let request = client
                .list_tables()
                .set_exclusive_start_table_name(last_evaluated_table_name.clone());

            match rt.block_on(async { request.send().await }) {
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
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(5), Constraint::Length(3)])
            .split(area);

        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Collections");

        if self.selected {
            block = block.border_style(Style::default().fg(SELECTED_COLOR));
        }

        let items: Vec<ListItem> = self
            .filtered_collections
            .iter()
            .map(|name| ListItem::new(name.clone()))
            .collect();

        let collection_list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(
            collection_list,
            layout[0],
            buf,
            &mut self.collections_list.state,
        );

        self.filter_input.render(layout[1], buf);
    }

    fn handle_event(&mut self, event: KeyEvent) {
        if !self.selected {
            match event.code {
                KeyCode::Char('c') => self.selected = true,
                _ => {}
            }
        } else if self.filter_input.active {
            self.filter_input.handle_event(event);
            self.apply_filter();
        } else {
            match event.code {
                KeyCode::Char('f') => {
                    self.filter_input.active = true;
                    self.filter_input.reset();
                }
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
        self.filter_input.active = false;
    }
}
