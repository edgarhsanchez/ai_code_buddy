use bevy::prelude::*;
use ratatui::layout::{Position, Rect};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum OverviewComponent {
    StartAnalysis,
    ViewReports,
    Settings,
    Credits,
    Help,
    Exit,
}

#[derive(Debug, Clone, Resource)]
pub struct OverviewWidgetState {
    pub selected_component: OverviewComponent,
    pub hovered_component: Option<OverviewComponent>,
    pub registered_components: HashMap<OverviewComponent, Rect>,
    pub repo_info: RepoInfo,
    pub show_help: bool,
}

#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub path: String,
    pub source_branch: String,
    pub target_branch: String,
    pub files_to_analyze: usize,
}

impl Default for OverviewWidgetState {
    fn default() -> Self {
        Self {
            selected_component: OverviewComponent::StartAnalysis,
            hovered_component: None,
            registered_components: HashMap::new(),
            show_help: false,
            repo_info: RepoInfo {
                path: ".".to_string(),
                source_branch: "main".to_string(),
                target_branch: "HEAD".to_string(),
                files_to_analyze: 0,
            },
        }
    }
}

impl OverviewWidgetState {
    pub fn is_over(&self, component: OverviewComponent, x: u16, y: u16) -> bool {
        if let Some(rect) = self.registered_components.get(&component) {
            rect.contains(Position { x, y })
        } else {
            false
        }
    }

    pub fn update_hover(&mut self, x: u16, y: u16) {
        self.hovered_component = None;
        for (component, rect) in &self.registered_components {
            if rect.contains(Position { x, y }) {
                self.hovered_component = Some(component.clone());
                break;
            }
        }
    }

    pub fn move_selection(&mut self, direction: SelectionDirection) {
        self.selected_component = match direction {
            SelectionDirection::Next => match self.selected_component {
                OverviewComponent::StartAnalysis => OverviewComponent::ViewReports,
                OverviewComponent::ViewReports => OverviewComponent::Settings,
                OverviewComponent::Settings => OverviewComponent::Credits,
                OverviewComponent::Credits => OverviewComponent::Help,
                OverviewComponent::Help => OverviewComponent::Exit,
                OverviewComponent::Exit => OverviewComponent::StartAnalysis,
            },
            SelectionDirection::Previous => match self.selected_component {
                OverviewComponent::StartAnalysis => OverviewComponent::Exit,
                OverviewComponent::ViewReports => OverviewComponent::StartAnalysis,
                OverviewComponent::Settings => OverviewComponent::ViewReports,
                OverviewComponent::Credits => OverviewComponent::Settings,
                OverviewComponent::Help => OverviewComponent::Credits,
                OverviewComponent::Exit => OverviewComponent::Help,
            },
        }
    }
}

#[derive(Debug)]
pub enum SelectionDirection {
    Next,
    Previous,
}
