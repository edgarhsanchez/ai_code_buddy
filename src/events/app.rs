use crate::bevy_states::app::AppState;
use bevy::prelude::*;

#[derive(Debug, Clone, Event)]
pub enum AppEvent {
    SwitchTo(AppState),
    Exit,
}
