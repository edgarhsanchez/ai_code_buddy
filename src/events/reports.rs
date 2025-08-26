use bevy::prelude::*;
use bevy_ratatui::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone, Event)]
pub enum ReportsEvent {
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
}
