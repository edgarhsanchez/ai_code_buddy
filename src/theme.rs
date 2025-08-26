use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
    pub text_primary: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue, 
            accent: Color::Magenta,
            background: Color::Black,
            error: Color::Red,
            warning: Color::Yellow,
            success: Color::Green,
            info: Color::Blue,
            text_primary: Color::White,
        }
    }
}

impl Theme {
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.secondary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error_style(&self) -> Style {
        Style::default()
            .fg(self.error)
            .add_modifier(Modifier::BOLD)
    }

    pub fn warning_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    pub fn info_style(&self) -> Style {
        Style::default()
            .fg(self.info)
    }

    pub fn selected_style(&self) -> Style {
        Style::default()
            .bg(self.primary)
            .fg(self.background)
            .add_modifier(Modifier::BOLD)
    }

    pub fn button_style(&self, pressed: bool) -> Style {
        if pressed {
            Style::default()
                .bg(self.accent)
                .fg(self.background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(self.primary)
        }
    }

    pub fn button_hover_style(&self) -> Style {
        Style::default()
            .bg(self.secondary)
            .fg(self.text_primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn button_normal_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::DIM)
    }

    pub fn primary_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
    }
}

pub static THEME: Theme = Theme {
    primary: Color::Cyan,
    secondary: Color::Blue,
    accent: Color::Magenta, 
    background: Color::Black,
    error: Color::Red,
    warning: Color::Yellow,
    success: Color::Green,
    info: Color::Blue,
    text_primary: Color::White,
};