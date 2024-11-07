use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{EMERALD, VIOLET};

use ratatui::style::Color;
use ratatui::widgets::{List, ListItem, ListState, StatefulWidget};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
};
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
    has_more: bool,
    list_state: ListState,
    selected_row: String,
}

impl DataBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_title(&mut self, new_title: &str) {
        self.title = new_title.to_string();
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
            Action::SelectDataMode => self.active = true,
            Action::SelectingRegion | Action::FilteringTables | Action::SelectTableMode => {
                self.active = false
            }
            Action::TransmitSelectedTable(table) => self.set_title(&table),
            Action::TransmitTableData(data, has_more) => {
                self.records = data;
                self.has_more = has_more;
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
