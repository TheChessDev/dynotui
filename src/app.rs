use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{
    action::Action,
    components::{
        collections_box::CollectionsBox, data_box::DataBox, data_detail_box::DataDetailBox,
        filter_input::FilterInput, loading::LoadingBox, region_box::AWSRegionBox, Component,
    },
    config::Config,
    data::{FetchRequest, FetchResponse},
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    fetch_tx: mpsc::Sender<FetchRequest>,
    fetch_rx: mpsc::Receiver<FetchResponse>,
    last_evaluated_key: Option<HashMap<String, AttributeValue>>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    View,
    Insert,
    FilterData,
    QueryData,
    SelectTable,
    SelectTableDataRow,
    ViewTableDataRowDetail,
}

impl App {
    pub fn new(
        tick_rate: f64,
        frame_rate: f64,
        fetch_tx: mpsc::Sender<FetchRequest>,
        fetch_rx: mpsc::Receiver<FetchResponse>,
    ) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let region = "us-east-1";
        let filter_collections_title = "Filter Tables";

        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![
                Box::new(CollectionsBox::new()),
                Box::new(DataBox::new()),
                Box::new(AWSRegionBox::new(region)),
                Box::new(FilterInput::new(filter_collections_title)),
                Box::new(LoadingBox::new()),
                Box::new(DataDetailBox::new()),
            ],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::View,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            fetch_rx,
            fetch_tx,
            last_evaluated_key: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        self.action_tx.send(Action::SelectTableMode)?;

        let action_tx = self.action_tx.clone();
        loop {
            while let Ok(response) = self.fetch_rx.try_recv() {
                match response {
                    FetchResponse::Tables(tables) => {
                        self.action_tx.send(Action::TransmitTables(tables))?;
                        self.action_tx.send(Action::Render)?;
                        self.action_tx.send(Action::StopLoading)?;
                    }
                    FetchResponse::TableData(data, has_more, last_evaluated_key) => {
                        self.last_evaluated_key = last_evaluated_key;
                        self.action_tx
                            .send(Action::TransmitTableData(data, has_more))?;
                        self.action_tx.send(Action::SelectDataMode)?;
                        self.action_tx.send(Action::Render)?;
                    }
                    FetchResponse::NextBatchTableData(data, has_more, last_evaluated_key) => {
                        self.last_evaluated_key = last_evaluated_key;
                        self.action_tx
                            .send(Action::TransmitNextBatcTableData(data, has_more))?;
                        self.action_tx.send(Action::Render)?;
                    }
                    FetchResponse::ApproximateTableDataCount(count) => {
                        self.action_tx
                            .send(Action::ApproximateTableDataCount(count))?;
                    }
                }
            }

            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();

        match self.mode {
            Mode::FilterData => {
                let Some(keymap) = self.config.keybindings.get(&self.mode) else {
                    return Ok(());
                };

                if let Some(action) = keymap.get(&vec![key]) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                } else if let Some(character) = self.get_char_from_key_event(key) {
                    action_tx.send(Action::NewFilterDataCharacter(character))?;
                }

                Ok(())
            }
            Mode::Insert => {
                let Some(keymap) = self.config.keybindings.get(&self.mode) else {
                    return Ok(());
                };

                if let Some(action) = keymap.get(&vec![key]) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                } else if let Some(character) = self.get_char_from_key_event(key) {
                    action_tx.send(Action::NewCharacter(character))?;
                }

                Ok(())
            }
            _ => {
                let Some(keymap) = self.config.keybindings.get(&self.mode) else {
                    return Ok(());
                };

                match keymap.get(&vec![key]) {
                    Some(action) => {
                        info!("Got action: {action:?}");
                        action_tx.send(action.clone())?;
                    }
                    _ => {
                        // If the key was not handled as a single key action,
                        // then consider it for multi-key combinations.
                        self.last_tick_key_events.push(key);

                        // Check for multi-key combinations
                        if let Some(action) = keymap.get(&self.last_tick_key_events) {
                            info!("Got action: {action:?}");
                            action_tx.send(action.clone())?;
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::FilterTableData => self.mode = Mode::FilterData,
                Action::QueryTableData => self.mode = Mode::QueryData,
                Action::EnterInsertMode => self.mode = Mode::Insert,
                Action::ExitInsertMode => self.mode = Mode::View,
                Action::SelectTableMode => self.mode = Mode::SelectTable,
                Action::SelectDataMode
                | Action::ExitFilterTableData
                | Action::ExitQueryTableData
                | Action::SubmitFilterDataText => self.mode = Mode::SelectTableDataRow,
                Action::ViewTableDataRowDetail => self.mode = Mode::ViewTableDataRowDetail,
                Action::FetchTables => {
                    self.fetch_tx.try_send(FetchRequest::Tables)?;
                }
                Action::FetchTableData(ref collection_name) => {
                    self.fetch_tx
                        .try_send(FetchRequest::GetApproximateItemCount(
                            collection_name.to_string(),
                        ))?;
                    self.fetch_tx
                        .try_send(FetchRequest::TableData(collection_name.to_string()))?;
                }
                Action::FetchMoreTableData(ref collection_name) => {
                    self.fetch_tx
                        .try_send(FetchRequest::GetApproximateItemCount(
                            collection_name.to_string(),
                        ))?;
                    self.fetch_tx.try_send(FetchRequest::NextBatchTableData(
                        collection_name.to_string(),
                        self.last_evaluated_key.clone(),
                    ))?;
                }
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }

    fn get_char_from_key_event(&self, key_event: KeyEvent) -> Option<char> {
        match key_event.code {
            KeyCode::Char(c) => Some(c),
            _ => None,
        }
    }
}
