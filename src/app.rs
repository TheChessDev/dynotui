use std::time::{Duration, Instant};

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_dynamodb::Client;
use ratatui::{
    buffer::Buffer,
    crossterm::{
        self,
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    },
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
    DefaultTerminal, Frame,
};
use tokio::time::sleep;

use crate::{
    components::{
        collections_box::CollectionsBox, data_box::DataBox, loading::LoadingBox,
        region_box::AWSRegionBox, MutableComponent,
    },
    message::Message,
};

pub struct App {
    mode: Mode,
    exit: bool,
    collections_box: CollectionsBox,
    aws_region_box: AWSRegionBox,
    data_box: DataBox,
    client: Client,
    loading_box: LoadingBox,
}

#[derive(Default)]
pub enum Mode {
    #[default]
    FilteringCollections,
    SelectingCollection,
    SelectingDataItem,
    SelectingRegion,
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        let region = "us-east-1";
        let region_provider = RegionProviderChain::default_provider().or_else(region);
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(region_provider)
            .load()
            .await;

        let client = Client::new(&config);

        let mut collections_box = CollectionsBox::new();
        collections_box.load_collections(&client).await;

        Ok(Self {
            exit: false,
            mode: Mode::SelectingCollection,
            collections_box,
            aws_region_box: AWSRegionBox::new(region),
            data_box: DataBox::new(),
            client,
            loading_box: LoadingBox::new(),
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let tick_rate = Duration::from_millis(50);
        let mut last_tick = Instant::now();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            self.handle_events(timeout).await?;

            if self.loading_box.active && last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_events(&mut self, timeout: Duration) -> anyhow::Result<()> {
        let mut messages = Vec::new();

        if crossterm::event::poll(timeout)? {
            let event = event::read()?;
            match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match self.mode {
                        Mode::SelectingCollection => {
                            self.collections_box
                                .handle_event(key_event, |msg| messages.push(msg));
                        }
                        Mode::FilteringCollections => {
                            self.collections_box
                                .filter_input
                                .handle_event(key_event, |msg| messages.push(msg));
                            self.collections_box.apply_filter();
                            self.collections_box.select_first();
                        }
                        Mode::SelectingDataItem => {
                            self.data_box
                                .handle_event(key_event, |msg| messages.push(msg));
                        }
                        Mode::SelectingRegion => {
                            self.aws_region_box
                                .handle_event(key_event, |msg| messages.push(msg));
                        }
                    }
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }

        for message in messages {
            self.process_message(message).await
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if !matches!(self.mode, Mode::FilteringCollections) {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit(),
                KeyCode::Char('/') => {
                    self.mode = Mode::FilteringCollections;
                    self.collections_box.filter_input.reset();
                }
                KeyCode::Char('i') => self.mode = Mode::SelectingDataItem,
                KeyCode::Char('t') => self.mode = Mode::SelectingCollection,
                KeyCode::Char('r') => self.mode = Mode::SelectingRegion,
                _ => {}
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub async fn process_message(&mut self, message: Message) {
        if message.should_trigger_loading() {
            if let Some(msg) = message.loading_message() {
                self.loading_box.set_message(msg);
            }

            self.loading_box.start_loading();

            self.on_tick();
        }

        match message {
            Message::CancelFilteringCollectionMode => {
                self.mode = Mode::SelectingCollection;
                self.collections_box.apply_filter();
            }
            Message::ApplyCollectionsFilter => self.mode = Mode::SelectingCollection,
            Message::FilterFromSelectingCollectionMode => self.mode = Mode::FilteringCollections,
            Message::SelectCollection(collection) => {
                self.loading_box.start_loading();
                self.on_tick();
                self.data_box.reset();

                sleep(Duration::from_secs(6)).await;

                if let Err(_error) = self.data_box.load_data(&self.client, &collection).await {
                    self.mode = Mode::SelectingCollection;
                    self.loading_box.end_loading();
                    return;
                }

                self.data_box.set_title(&collection);
                self.mode = Mode::SelectingDataItem;
                self.loading_box.start_loading();
            }
            Message::LoadMoreData => {
                if let Err(_error) = self
                    .data_box
                    .load_data(&self.client, &self.collections_box.selected_collection)
                    .await
                {
                    self.mode = Mode::SelectingCollection;
                }
            }
        }
    }

    fn on_tick(&mut self) {
        self.loading_box.on_tick();
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(layout[0]);

        let left_col_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(columns[0]);

        self.loading_box.render(layout[1], buf, false);

        self.collections_box.render(
            left_col_layout[1],
            buf,
            matches!(self.mode, Mode::SelectingCollection),
        );

        self.aws_region_box.render(
            left_col_layout[0],
            buf,
            matches!(self.mode, Mode::SelectingRegion),
        );

        self.data_box.render(
            columns[1],
            buf,
            matches!(self.mode, Mode::SelectingDataItem),
        );

        self.collections_box.filter_input.render(
            left_col_layout[2],
            buf,
            matches!(self.mode, Mode::FilteringCollections),
        )
    }
}
