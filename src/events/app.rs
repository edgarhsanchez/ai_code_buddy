use bevy::prelude::*;
use crate::bevy_states::app::AppState;

#[derive(Debug, Clone, Event)]
pub enum AppEvent {
    SwitchTo(AppState),
    Exit,
}