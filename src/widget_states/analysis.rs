use bevy::prelude::*;
use crate::core::review::Review;

#[derive(Debug, Clone, Resource)]
pub struct AnalysisWidgetState {
    pub is_analyzing: bool,
    pub progress: f64,
    pub current_file: String,
    pub review: Option<Review>,
    pub selected_issue: usize,
}

impl Default for AnalysisWidgetState {
    fn default() -> Self {
        Self {
            is_analyzing: false,
            progress: 0.0,
            current_file: String::new(),
            review: None,
            selected_issue: 0,
        }
    }
}

impl AnalysisWidgetState {
    pub fn start_analysis(&mut self) {
        self.is_analyzing = true;
        self.progress = 0.0;
        self.current_file.clear();
        self.review = None;
    }
    
    pub fn update_progress(&mut self, progress: f64, current_file: String) {
        self.progress = progress;
        self.current_file = current_file;
    }
    
    pub fn complete_analysis(&mut self, review: Review) {
        self.is_analyzing = false;
        self.progress = 100.0;
        self.review = Some(review);
        self.selected_issue = 0;
    }
    
    pub fn move_issue_selection(&mut self, direction: i32) {
        if let Some(review) = &self.review {
            if !review.issues.is_empty() {
                let new_selection = (self.selected_issue as i32 + direction)
                    .max(0)
                    .min(review.issues.len() as i32 - 1) as usize;
                self.selected_issue = new_selection;
            }
        }
    }
}