use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
    DefaultTerminal, Frame,
};

use crate::components::{collections_box::CollectionsBox, region_box::AWSRegionBox, Component};

pub struct App {
    mode: Mode,
    exit: bool,
    collections_box: CollectionsBox,
    aws_region_box: AWSRegionBox,
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
        Ok(Self {
            exit: false,
            mode: Mode::Home,
            collections_box: CollectionsBox::new(),
            aws_region_box: AWSRegionBox::new(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.collections_box.handle_event(key_event);
                self.aws_region_box.handle_event(key_event);
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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        let left_col_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(5),
                Constraint::Percentage(85),
                Constraint::Percentage(10),
            ])
            .split(layout[0]);

        let mut filter_collections_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Filter");

        if let Mode::FilteringCollections = self.mode {
            filter_collections_block = filter_collections_block
                .clone()
                .border_style(Style::default().fg(Color::Green));
        }

        self.collections_box.render(left_col_layout[1], buf);
        self.aws_region_box.render(left_col_layout[0], buf);

        Paragraph::new("")
            .block(filter_collections_block)
            .render(left_col_layout[2], buf);

        Paragraph::new("")
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Data"),
            )
            .render(layout[1], buf);
    }
}
