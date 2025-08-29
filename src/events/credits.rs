use bevy::prelude::*;
use bevy_ratatui::event::{KeyEvent, MouseEvent};

#[derive(Event)]
pub enum CreditsEvent {
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
}
