use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::style::Color;
use ratatui::widgets::{
    HighlightSpacing, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState,
    StatefulWidget,
};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use symbols::scrollbar;
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
    scroll_bar_state: ScrollbarState,
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

    fn update_scroll_pos(&mut self, pos: usize) {
        self.scroll_bar_state = self.scroll_bar_state.position(pos);
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

    fn set_selected(&mut self) -> bool {
        if self.list_state.selected().is_none() {
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

    fn select_first_if_needed(&mut self) {
        if self.list_state.selected().is_none() {
            self.select_first();
        }
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
            Action::SelectTableMode => {
                self.active = true;
                let command_ref = self.command_tx.as_ref().unwrap();

                if self.collections.is_empty() {
                    command_ref.send(Action::StartLoading("Fetching Tables".to_string()))?;
                }

                command_ref.send(Action::FetchTables)?;
            }
            Action::FilteringTables
            | Action::SelectingRegion
            | Action::SelectDataMode
            | Action::ViewTableDataRowDetail => {
                self.active = false;
                self.list_state.select(None);
            }
            Action::TransmitSubmittedText(text) => {
                self.filter_text = text.clone();
                self.select_first_if_needed();
                self.apply_filter();
            }
            Action::TransmitTables(tables) => {
                self.collections = tables;
                self.select_first_if_needed();
                self.apply_filter();
            }
            Action::SelectTablePrev => {
                self.select_previous();
            }
            Action::SelectTableNext => {
                self.select_next();
            }
            Action::SelectTableScrollUp => {
                self.scroll_up();
            }
            Action::SelectTableScrollDown => {
                self.scroll_down();
            }
            Action::SelectTableFirst => {
                self.select_first();
            }
            Action::SelectTableLast => {
                self.select_last();
            }
            Action::SelectTable => {
                self.set_selected();
                let command_ref = self.command_tx.as_ref().unwrap();

                command_ref.send(Action::StartLoading("Fetching Table Data".to_string()))?;

                command_ref.send(Action::TransmitSelectedTable(
                    self.selected_collection.clone(),
                ))?;

                command_ref.send(Action::FetchTableData(self.selected_collection.clone()))?;
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

        self.scroll_bar_state = self.scroll_bar_state.content_length(items.len());

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(None)
            .track_symbol(None)
            .end_symbol(None);

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

        StatefulWidget::render(
            scrollbar,
            middle_left.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            frame.buffer_mut(),
            &mut self.scroll_bar_state,
        );

        Ok(())
    }
}
