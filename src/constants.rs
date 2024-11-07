use ratatui::style::{
    palette::tailwind::{EMERALD, VIOLET},
    Color, Modifier, Style,
};

pub const ACTIVE_PANE_COLOR: Color = EMERALD.c300;

pub const ROW_HOVER_COLOR: Color = VIOLET.c600;

pub const LIST_ITEM_SELECTED_STYLE: Style = Style::new()
    .bg(ROW_HOVER_COLOR)
    .add_modifier(Modifier::BOLD);
