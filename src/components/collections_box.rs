use aws_sdk_dynamodb::Client;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Color, Modifier};
use ratatui::widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::message::Message;

use super::filter_input::FilterInput;
use super::{MutableComponent, ACTIVE_PANE_COLOR, ROW_HOVER_COLOR};

const SELECTED_STYLE: Style = Style::new()
    .bg(ROW_HOVER_COLOR)
    .add_modifier(Modifier::BOLD);

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
            filter_input: FilterInput::new("Filter Tables"),
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

    pub async fn load_collections(&mut self, client: &Client) {
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
        self.collections_list.state.select(None);
    }

    fn select_next(&mut self) {
        self.collections_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.collections_list.state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.collections_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.collections_list.state.select_last();
    }

    fn scroll_up(&mut self) {
        self.collections_list.state.scroll_up_by(5);
    }

    fn scroll_down(&mut self) {
        self.collections_list.state.scroll_down_by(5);
    }

    fn set_selected(&mut self) -> bool {
        if let None = self.collections_list.state.selected() {
            return false;
        }

        let col_indx = self.collections_list.state.selected().unwrap();
        let new_col_name = self.filtered_collections[col_indx].to_string();

        if new_col_name == self.selected_collection {
            return false;
        }

        self.selected_collection = new_col_name;

        return true;
    }
}

impl MutableComponent for CollectionsBox {
    fn render(&mut self, area: Rect, buf: &mut Buffer, active: bool) {
        self.selected = active;

        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Tables");

        if self.selected {
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
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(collection_list, area, buf, &mut self.collections_list.state);
    }

    fn handle_event<F>(&mut self, event: KeyEvent, send_message: F)
    where
        F: FnOnce(Message),
    {
        match event.code {
            KeyCode::Char('/') => send_message(Message::FilterFromSelectingCollectionMode),
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                let new_selection = self.set_selected();

                if new_selection {
                    send_message(Message::SelectCollection(self.selected_collection.clone()))
                }
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
        self.selected = false;
    }
}
