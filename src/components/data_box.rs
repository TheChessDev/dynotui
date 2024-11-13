use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{EMERALD, VIOLET};

use clipboard::{ClipboardContext, ClipboardProvider};
use ratatui::style::Color;
use ratatui::widgets::{List, ListItem, ListState, Padding, Paragraph, StatefulWidget};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
};
use style::palette::material::INDIGO;
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
    collection_name: String,
    fetching: bool,
    aprox_count: i64,
}

impl DataBox {
    pub fn new() -> Self {
        Self {
            title: "Data".to_string(),
            ..Self::default()
        }
    }

    pub fn set_title(&mut self, new_title: &str) {
        self.title = new_title.to_string();
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

    fn copy_selected_row_to_clipboard(&self) {
        if let Some(i) = self.list_state.selected() {
            let selected_row = &self.records[i];

            let mut ctx: ClipboardContext =
                ClipboardProvider::new().expect("Failed to access clipboard");
            ctx.set_contents(selected_row.clone())
                .expect("Failed to copy to clipboard");
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
            .records
            .iter()
            .map(|record| ListItem::new(record.clone()))
            .collect();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::new().bg(VIOLET.c600).add_modifier(Modifier::BOLD));

        StatefulWidget::render(list, right, frame.buffer_mut(), &mut self.list_state);

        let status_text = format!(
            "Fetched {} Items (Scanned: {})",
            self.records.len(),
            self.aprox_count
        );

        if self.active {
            Paragraph::new(status_text)
                .block(Block::default().padding(Padding::horizontal(2)))
                .style(Style::new().fg(INDIGO.c700))
                .render(bottom_right, frame.buffer_mut());
        }

        Ok(())
    }
}
