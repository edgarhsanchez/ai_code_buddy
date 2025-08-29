use bevy::prelude::*;
use ratatui::layout::{Position, Rect};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CreditsComponent {
    BackToOverview,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug, Clone, Resource)]
pub struct CreditsWidgetState {
    #[allow(dead_code)]
    pub selected_component: CreditsComponent,
    #[allow(dead_code)]
    pub hovered_component: Option<CreditsComponent>,
    pub registered_components: HashMap<CreditsComponent, Rect>,
    pub scroll_offset: usize,
    pub total_lines: usize,
}

impl Default for CreditsWidgetState {
    fn default() -> Self {
        Self {
            selected_component: CreditsComponent::BackToOverview,
            hovered_component: None,
            registered_components: HashMap::new(),
            scroll_offset: 0,
            total_lines: 0,
        }
    }
}

impl CreditsWidgetState {
    pub fn is_over(&self, component: CreditsComponent, x: u16, y: u16) -> bool {
        if let Some(rect) = self.registered_components.get(&component) {
            rect.contains(Position { x, y })
        } else {
            false
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(5);
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.total_lines.saturating_sub(20) {
            self.scroll_offset += 5;
        }
    }
}
