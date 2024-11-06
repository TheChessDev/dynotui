use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::{
        palette::tailwind::{EMERALD, VIOLET},
        Color,
    },
};

use crate::app::Message;

pub mod collections_box;
pub mod data_box;
pub mod filter_input;
pub mod loading;
pub mod region_box;

pub trait MutableComponent {
    /// Render the component
    fn render(&mut self, area: Rect, buf: &mut Buffer, active: bool);

    /// Handle an input event, such as key press
    fn handle_event<F>(&mut self, event: KeyEvent, send_message: F)
    where
        F: FnOnce(Message);

    /// Reset the component to its initial state (optional)
    fn reset(&mut self);
}

pub const ACTIVE_PANE_COLOR: Color = EMERALD.c300;

pub const ROW_HOVER_COLOR: Color = VIOLET.c600;
