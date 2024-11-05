use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::{palette::tailwind::EMERALD, Color},
};

pub mod collections_box;
pub mod data_box;
pub mod filter_input;
pub mod region_box;

pub trait Component {
    /// Render the component
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Handle an input event, such as key press
    fn handle_event(&mut self, event: KeyEvent);

    /// Reset the component to its initial state (optional)
    fn reset(&mut self);
}

pub trait MutableComponent {
    /// Render the component
    fn render(&mut self, area: Rect, buf: &mut Buffer);

    /// Handle an input event, such as key press
    fn handle_event(&mut self, event: KeyEvent);

    /// Reset the component to its initial state (optional)
    fn reset(&mut self);
}

pub const SELECTED_COLOR: Color = EMERALD.c300;
