use ai_code_buddy::{
    app_events_handler,
    args::{Args, OutputFormat},
    bevy_states::app::AppState,
    events::app::AppEvent,
    initialize_app,
};
use bevy::{app::App, ecs::event::Events, prelude::*, state::app::StatesPlugin};

#[test]
fn test_initialize_app() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>();

    let args = Args {
        repo_path: "/test/repo".to_string(),
        source_branch: "main".to_string(),
        target_branch: "feature".to_string(),
        cli_mode: false,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: false,
    };

    app.insert_resource(args);

    // Test that we can schedule the system
    app.add_systems(Startup, initialize_app);

    // Run startup systems
    app.update();

    // Check that the state was set to Overview
    let state = app.world().get_resource::<State<AppState>>().unwrap();
    assert_eq!(*state.get(), AppState::Overview);
}

#[test]
fn test_app_events_handler_switch_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AppEvent>();

    // Set initial state
    let mut next_state = app
        .world_mut()
        .get_resource_mut::<NextState<AppState>>()
        .unwrap();
    next_state.set(AppState::Overview);

    // Update to apply the initial state
    app.update();

    // Send switch state event
    let mut events = app
        .world_mut()
        .get_resource_mut::<Events<AppEvent>>()
        .unwrap();
    events.send(AppEvent::SwitchTo(AppState::Analysis));

    // Add the system
    app.add_systems(Update, app_events_handler);

    // Run the system
    app.update();

    // Update again to apply the state change
    app.update();

    // Check that the state changed
    let state = app.world().get_resource::<State<AppState>>().unwrap();
    assert_eq!(*state.get(), AppState::Analysis);
}

#[test]
fn test_app_events_handler_exit() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AppEvent>();

    // Send exit event
    let mut events = app
        .world_mut()
        .get_resource_mut::<Events<AppEvent>>()
        .unwrap();
    events.send(AppEvent::Exit);

    // Add the system
    app.add_systems(Update, app_events_handler);

    // Run the system
    app.update();

    // Check that exit event was sent
    let exit_events = app.world().get_resource::<Events<AppExit>>().unwrap();
    assert!(!exit_events.is_empty());
}

#[test]
fn test_app_events_handler_multiple_events() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AppEvent>();

    // Set initial state
    let mut next_state = app
        .world_mut()
        .get_resource_mut::<NextState<AppState>>()
        .unwrap();
    next_state.set(AppState::Overview);
    app.update();

    // Send multiple events
    let mut events = app
        .world_mut()
        .get_resource_mut::<Events<AppEvent>>()
        .unwrap();
    events.send(AppEvent::SwitchTo(AppState::Analysis));
    events.send(AppEvent::SwitchTo(AppState::Reports));
    events.send(AppEvent::Exit);

    // Add the system
    app.add_systems(Update, app_events_handler);

    // Run the system
    app.update();

    // Update again to apply state changes
    app.update();

    // Check that the last state change took effect (Reports)
    let state = app.world().get_resource::<State<AppState>>().unwrap();
    assert_eq!(*state.get(), AppState::Reports);

    // Check that exit event was sent
    let exit_events = app.world().get_resource::<Events<AppExit>>().unwrap();
    assert!(!exit_events.is_empty());
}

#[test]
fn test_state_transitions() {
    let states = vec![AppState::Overview, AppState::Analysis, AppState::Reports];

    for target_state in states {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .init_state::<AppState>()
            .add_event::<AppEvent>();

        // Set initial state
        let mut next_state = app
            .world_mut()
            .get_resource_mut::<NextState<AppState>>()
            .unwrap();
        next_state.set(AppState::Overview);
        app.update();

        // Send switch event
        let mut events = app
            .world_mut()
            .get_resource_mut::<Events<AppEvent>>()
            .unwrap();
        events.send(AppEvent::SwitchTo(target_state));

        // Add and run the system
        app.add_systems(Update, app_events_handler);
        app.update();
        app.update(); // Apply state change

        // Check that the state changed
        let current_state = app.world().get_resource::<State<AppState>>().unwrap();
        assert_eq!(*current_state.get(), target_state);
    }
}

#[test]
fn test_initialize_app_with_different_args() {
    let test_cases = vec![
        Args {
            repo_path: "/home/user/project".to_string(),
            source_branch: "develop".to_string(),
            target_branch: "master".to_string(),
            cli_mode: false,
            verbose: true,
            show_credits: false,
            output_format: OutputFormat::Detailed,
            exclude_patterns: vec!["*.tmp".to_string()],
            include_patterns: vec!["src/**".to_string()],
            use_gpu: true,
            force_cpu: false,
        },
        Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: OutputFormat::Json,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: true,
        },
    ];

    for args in test_cases {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .init_state::<AppState>()
            .insert_resource(args);

        // Add and run the system
        app.add_systems(Startup, initialize_app);
        app.update();

        // Check that the state was set to Overview
        let state = app.world().get_resource::<State<AppState>>().unwrap();
        assert_eq!(*state.get(), AppState::Overview);
    }
}

#[test]
fn test_app_events_handler_no_events() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AppEvent>();

    // Set initial state
    let mut state = app
        .world_mut()
        .get_resource_mut::<NextState<AppState>>()
        .unwrap();
    state.set(AppState::Overview);
    app.update();

    // Don't send any events
    let initial_state = app.world().get_resource::<State<AppState>>().unwrap();
    let initial_state_value = *initial_state.get();

    // Add and run the system
    app.add_systems(Update, app_events_handler);
    app.update();

    // Check that the state didn't change
    let final_state = app.world().get_resource::<State<AppState>>().unwrap();
    assert_eq!(*final_state.get(), initial_state_value);

    // Check that no exit event was sent
    let exit_events = app.world().get_resource::<Events<AppExit>>().unwrap();
    assert!(exit_events.is_empty());
}

#[test]
fn test_event_system_setup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AppEvent>();

    let args = Args {
        repo_path: "/test/repo".to_string(),
        source_branch: "main".to_string(),
        target_branch: "feature".to_string(),
        cli_mode: false,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: false,
    };

    app.insert_resource(args);

    // Verify that the event systems can be added without panicking
    app.add_systems(Startup, initialize_app)
        .add_systems(Update, app_events_handler);

    // Run a few updates
    for _ in 0..3 {
        app.update();
    }

    // If we get here without panicking, the test passes
    assert!(true);
}
