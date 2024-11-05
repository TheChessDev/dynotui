use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
    DefaultTerminal, Frame,
};

use crate::components::{
    collections_box::CollectionsBox, data_box::DataBox, region_box::AWSRegionBox, MutableComponent,
};

pub struct App {
    mode: Mode,
    exit: bool,
    collections_box: CollectionsBox,
    aws_region_box: AWSRegionBox,
    data_box: DataBox,
}

#[derive(Default)]
pub enum Mode {
    #[default]
    FilteringCollections,
    SelectingCollection,
    SelectingDataItem,
    SelectingRegion,
}

pub enum Message {
    ApplyCollectionsFilter,
    CancelFilteringCollectionMode,
    FilterFromSelectingCollectionMode,
    SelectCollection(String),
}

impl App {
    pub fn new() -> io::Result<Self> {
        let region = "us-east-1";
        let mut collections_box = CollectionsBox::new();
        collections_box.load_collections(region);

        Ok(Self {
            exit: false,
            mode: Mode::SelectingCollection,
            collections_box,
            aws_region_box: AWSRegionBox::new(region),
            data_box: DataBox::new(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        let mut messages = Vec::new();

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

        for message in messages {
            self.process_message(message);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if !matches!(self.mode, Mode::FilteringCollections) {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit(),
                KeyCode::Char('f') => {
                    self.mode = Mode::FilteringCollections;
                    self.data_box.reset();
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

    pub fn process_message(&mut self, message: Message) {
        match message {
            Message::CancelFilteringCollectionMode => {
                self.mode = Mode::SelectingCollection;
                self.collections_box.apply_filter();
            }
            Message::ApplyCollectionsFilter => self.mode = Mode::SelectingCollection,
            Message::FilterFromSelectingCollectionMode => self.mode = Mode::FilteringCollections,
            Message::SelectCollection(collection) => {
                self.data_box.set_title(&collection);
                self.mode = Mode::SelectingDataItem;
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        let left_col_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(layout[0]);

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

        self.data_box
            .render(layout[1], buf, matches!(self.mode, Mode::SelectingDataItem));

        self.collections_box.filter_input.render(
            left_col_layout[2],
            buf,
            matches!(self.mode, Mode::FilteringCollections),
        )
    }
}
