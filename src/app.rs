use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
    DefaultTerminal, Frame,
};

use crate::components::{
    collections_box::CollectionsBox, data_box::DataBox, region_box::AWSRegionBox, Component,
    MutableComponent,
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
    Home,
    FilteringCollections,
    SelectingCollection,
    SelectingDataItem,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let mut collections_box = CollectionsBox::new();
        collections_box.load_collections("us-east-1");

        Ok(Self {
            exit: false,
            mode: Mode::Home,
            collections_box,
            aws_region_box: AWSRegionBox::new(),
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
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.collections_box.handle_event(key_event);
                self.aws_region_box.handle_event(key_event);
                self.data_box.handle_event(key_event);
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('f') => self.mode = Mode::FilteringCollections,
            KeyCode::Char('d') => self.mode = Mode::SelectingDataItem,
            KeyCode::Char('c') => self.mode = Mode::SelectingCollection,
            KeyCode::Esc => self.mode = Mode::Home,
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
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
            .constraints(vec![Constraint::Percentage(5), Constraint::Min(0)])
            .split(layout[0]);

        self.collections_box.render(left_col_layout[1], buf);

        self.aws_region_box.render(left_col_layout[0], buf);
        self.data_box.render(layout[1], buf);
    }
}
