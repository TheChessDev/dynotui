use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect};

pub mod collections_box;

pub trait Component {
    /// Render the component
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Handle an input event, such as key press
    fn handle_event(&mut self, event: KeyEvent);

    /// Reset the component to its initial state (optional)
    fn reset(&mut self);
}
