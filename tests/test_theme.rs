use ai_code_buddy::theme::Theme;
use ratatui::style::{Color, Modifier};

#[test]
fn test_theme_default() {
    let theme = Theme::default();

    assert_eq!(theme.primary, Color::Cyan);
    assert_eq!(theme.secondary, Color::Blue);
    assert_eq!(theme.accent, Color::Magenta);
    assert_eq!(theme.background, Color::Black);
    assert_eq!(theme.error, Color::Red);
    assert_eq!(theme.warning, Color::Yellow);
    assert_eq!(theme.success, Color::Green);
    assert_eq!(theme.info, Color::Blue);
    assert_eq!(theme.text_primary, Color::White);
}

#[test]
fn test_title_style() {
    let theme = Theme::default();
    let style = theme.title_style();

    assert_eq!(style.fg, Some(Color::Cyan));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_header_style() {
    let theme = Theme::default();
    let style = theme.header_style();

    assert_eq!(style.fg, Some(Color::Blue));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_success_style() {
    let theme = Theme::default();
    let style = theme.success_style();

    assert_eq!(style.fg, Some(Color::Green));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_error_style() {
    let theme = Theme::default();
    let style = theme.error_style();

    assert_eq!(style.fg, Some(Color::Red));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_warning_style() {
    let theme = Theme::default();
    let style = theme.warning_style();

    assert_eq!(style.fg, Some(Color::Yellow));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_info_style() {
    let theme = Theme::default();
    let style = theme.info_style();

    assert_eq!(style.fg, Some(Color::Blue));
}

#[test]
fn test_selected_style() {
    let theme = Theme::default();
    let style = theme.selected_style();

    assert_eq!(style.bg, Some(Color::Cyan));
    assert_eq!(style.fg, Some(Color::Black));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_button_style_normal() {
    let theme = Theme::default();
    let style = theme.button_style(false);

    assert_eq!(style.fg, Some(Color::Cyan));
}

#[test]
fn test_button_style_pressed() {
    let theme = Theme::default();
    let style = theme.button_style(true);

    assert_eq!(style.bg, Some(Color::Magenta));
    assert_eq!(style.fg, Some(Color::Black));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_button_hover_style() {
    let theme = Theme::default();
    let style = theme.button_hover_style();

    assert_eq!(style.bg, Some(Color::Blue));
    assert_eq!(style.fg, Some(Color::White));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_button_normal_style() {
    let theme = Theme::default();
    let style = theme.button_normal_style();

    assert_eq!(style.fg, Some(Color::Cyan));
    assert!(style.add_modifier.contains(Modifier::DIM));
}

#[test]
fn test_primary_style() {
    let theme = Theme::default();
    let style = theme.primary_style();

    assert_eq!(style.fg, Some(Color::Cyan));
}

#[test]
fn test_custom_theme() {
    let custom_theme = Theme {
        primary: Color::Red,
        secondary: Color::Green,
        accent: Color::Blue,
        background: Color::White,
        error: Color::Magenta,
        warning: Color::Cyan,
        success: Color::Yellow,
        info: Color::Black,
        text_primary: Color::Gray,
    };

    assert_eq!(custom_theme.primary, Color::Red);
    assert_eq!(custom_theme.text_primary, Color::Gray);

    let style = custom_theme.title_style();
    assert_eq!(style.fg, Some(Color::Red));
}
