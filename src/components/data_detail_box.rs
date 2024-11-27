use std::collections::HashMap;

use clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{palette::tailwind::EMERALD, Color, Style},
    symbols::scrollbar,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::{action::Action, config::Config};

use super::Component;

#[derive(Clone, Debug)]
struct TreeNode {
    key: String,
    value: Value,
    depth: usize,
    expanded: bool,
    path: Vec<String>,
}

pub struct DataDetailBox {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    active: bool,
    title: String,
    row: String,
    tree: Vec<TreeNode>,
    vertical_scroll: usize,
    horizontal_scroll: usize,
    vertical_scroll_state: ScrollbarState,
    horizontal_scroll_state: ScrollbarState,
    selected_index: usize,
    expanded_states: HashMap<Vec<String>, bool>,
}

impl DataDetailBox {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            active: false,
            title: "JSON Viewer".to_string(),
            row: "".to_string(),
            tree: vec![],
            vertical_scroll: 0,
            horizontal_scroll: 0,
            horizontal_scroll_state: ScrollbarState::default(),
            vertical_scroll_state: ScrollbarState::default(),
            selected_index: 0,
            expanded_states: HashMap::new(),
        }
    }

    fn json_to_tree(&mut self, json: &Value, depth: usize, path: Vec<String>) -> Vec<TreeNode> {
        let mut nodes = Vec::new();

        match json {
            Value::Object(map) => {
                for (key, value) in map {
                    let new_path = [path.clone(), vec![key.clone()]].concat();
                    let expanded = *self.expanded_states.get(&new_path).unwrap_or(&false);

                    nodes.push(TreeNode {
                        key: key.clone(),
                        value: value.clone(),
                        depth,
                        expanded,
                        path: new_path.clone(),
                    });

                    if expanded {
                        nodes.extend(self.json_to_tree(value, depth + 1, new_path));
                    }
                }
            }
            Value::Array(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    let new_path = [path.clone(), vec![index.to_string()]].concat();
                    let expanded = *self.expanded_states.get(&new_path).unwrap_or(&false);

                    nodes.push(TreeNode {
                        key: index.to_string(),
                        value: value.clone(),
                        depth,
                        expanded,
                        path: new_path.clone(),
                    });

                    if expanded {
                        nodes.extend(self.json_to_tree(value, depth + 1, new_path));
                    }
                }
            }
            _ => {
                nodes.push(TreeNode {
                    key: "".to_string(),
                    value: json.clone(),
                    depth,
                    expanded: false,
                    path,
                });
            }
        }

        nodes
    }

    fn toggle_node(&mut self, path: &[String]) {
        if let Some(node) = self.tree.iter_mut().find(|node| node.path == path) {
            node.expanded = !node.expanded;
            self.expanded_states.insert(path.to_vec(), node.expanded); // Update expanded state

            // Automatically adjust scroll to center the toggled node
            let index = self
                .tree
                .iter()
                .position(|n| n.path == path)
                .unwrap_or(self.selected_index);
            self.center_scroll_on(index);
        }

        // Rebuild the tree to reflect the updated expanded state
        if let Ok(json) = self.parse_json() {
            self.tree = self.json_to_tree(&json, 0, vec![]);
        }
    }

    fn center_scroll_on(&mut self, index: usize) {
        let visible_area = 10;

        match index {
            i if i > self.vertical_scroll + visible_area / 2 => {
                // Scroll down to center
                self.vertical_scroll = i.saturating_sub(visible_area / 2);
            }
            i if i < self.vertical_scroll + visible_area / 2 => {
                // Scroll up to center
                self.vertical_scroll = i.saturating_sub(visible_area / 2);
            }
            _ => {
                // Index is already centered; no need to adjust
            }
        }
    }

    fn get_visible_nodes(&self) -> Vec<&TreeNode> {
        let mut visible_nodes = Vec::new();

        for node in &self.tree {
            visible_nodes.push(node);

            // If the node is expanded, ensure its children are included
            if !node.expanded {
                // Skip children by breaking out of the loop for non-expanded nodes
                continue;
            }
        }

        visible_nodes
    }

    fn render_tree(&mut self) -> Vec<Line> {
        let mut lines = Vec::new();

        let selected_index = self.selected_index;

        for (index, node) in self.get_visible_nodes().iter().enumerate() {
            let indent = " ".repeat(node.depth * 2);
            let line_content = if matches!(node.value, Value::Object(_) | Value::Array(_)) {
                format!(
                    "{}{} {}",
                    indent,
                    if node.expanded { "▼" } else { "▶" },
                    node.key
                )
            } else {
                format!("{}{}: {}", indent, node.key, node.value)
            };

            // Highlight the selected node
            let style = if index == selected_index {
                Style::default().fg(Color::White).bg(Color::Blue)
            } else {
                Style::default()
            };

            lines.push(Line::from(Span::styled(line_content, style)));
        }

        lines
    }

    fn parse_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.row)
    }

    fn copy_selected_row_to_clipboard(&self) {
        let mut ctx: ClipboardContext =
            ClipboardProvider::new().expect("Failed to access clipboard");

        ctx.set_contents(self.row.clone())
            .expect("Failed to copy to clipboard");
    }
}

impl Component for DataDetailBox {
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
            Action::ViewTableDataRowDetail => {
                self.active = true;
                self.horizontal_scroll = 0;
                self.vertical_scroll = 0;
            }
            Action::SelectingRegion
            | Action::FilteringTables
            | Action::SelectTableMode
            | Action::SelectDataMode => self.active = false,

            Action::ViewTableDataRowScrollUp => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(10);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
                self.selected_index = self.vertical_scroll;
            }
            Action::ViewTableDataRowScrollDown => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(10);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
                self.selected_index = self.vertical_scroll;
            }
            Action::ViewTableDataRowScrollLeft => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                self.horizontal_scroll_state = self
                    .horizontal_scroll_state
                    .position(self.horizontal_scroll);
            }
            Action::ViewTableDataRowScrollRight => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
                self.horizontal_scroll_state = self
                    .horizontal_scroll_state
                    .position(self.horizontal_scroll);
            }
            Action::TransmitSelectedTableDataRow(row) => {
                self.row = row.clone();
                if let Ok(json) = self.parse_json() {
                    self.tree = self.json_to_tree(&json, 0, vec![]);
                }
            }
            Action::ViewTableDataRowToggleNode => {
                let index = self.selected_index;
                if let Some(node) = self.get_visible_nodes().get(index) {
                    let path = node.path.clone();
                    self.toggle_node(&path);
                    self.command_tx.as_ref().unwrap().send(Action::Render)?;
                }
            }
            Action::ViewTableDataRowNavigateUp => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            Action::ViewTableDataRowNavigateDown => {
                self.selected_index =
                    (self.selected_index + 1).min(self.get_visible_nodes().len() - 1);
            }
            Action::ExitViewTableDataRowMode => {
                self.command_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::SelectDataMode)?;
            }
            Action::ViewTableDataRowCopyToClipboard => {
                self.copy_selected_row_to_clipboard();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        let [_, y_middle, _] = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .areas(area);

        let [_, middle, _] = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .areas(y_middle);

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(EMERALD.c300))
            .style(Style::new().bg(Color::Black))
            .title(self.title.clone());

        let vertical_scroll = self.vertical_scroll;
        let horizontal_scroll = self.horizontal_scroll;

        let lines = self.render_tree();

        let longest_line_width = lines.iter().map(|line| line.width()).max().unwrap_or(0);
        let total_lines = lines.len();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((vertical_scroll as u16, horizontal_scroll as u16));

        frame.render_widget(Clear, middle);
        frame.render_widget(paragraph, middle);

        self.vertical_scroll_state = self.vertical_scroll_state.content_length(total_lines);

        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .content_length(longest_line_width);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
            middle.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.vertical_scroll_state,
        );

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .symbols(scrollbar::HORIZONTAL)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
            middle.inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
            &mut self.horizontal_scroll_state,
        );

        Ok(())
    }
}
