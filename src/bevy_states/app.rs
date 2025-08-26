use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum AppState {
    #[default]
    Overview,
    Analysis,
    Reports,
}
