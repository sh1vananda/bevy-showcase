use bevy::prelude::*;

mod components;
mod combat;
mod overworld;
mod effects;

use components::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dungeon Gauntlet".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .insert_resource(CurrentBattle {
            enemy_entity: Entity::PLACEHOLDER,
            phase: BattlePhase::Intro,
            phase_timer: Timer::from_seconds(0.0, TimerMode::Once),
            player_defended: false,
            combo_count: 0,
        })
        .insert_resource(GameProgress {
            current_room: 0,
            rooms_cleared: 0,
            total_rooms: TOTAL_ROOMS,
        })
        .insert_resource(BulletSpawner {
            timer: Timer::from_seconds(0.7, TimerMode::Repeating),
            pattern: 0,
        })
        .add_systems(Startup, (setup_camera, setup_world, setup_ui))
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(Update, main_menu_input.run_if(in_state(GameState::MainMenu)))
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        .add_systems(
            Update,
            (
                overworld::player_movement,
                overworld::check_room_transition,
                overworld::check_exit_door,
                overworld::camera_follow,
            )
                .run_if(in_state(GameState::Overworld)),
        )
        .add_systems(OnEnter(GameState::Battle), (combat::setup_battle, combat::freeze_camera))
        .add_systems(
            Update,
            (
                combat::battle_phase_system,
                combat::update_attack_indicator,
                combat::player_turn_input,
                combat::bullet_hell_player_movement,
                combat::update_telegraph,
                combat::spawn_bullet_patterns,
                combat::update_bullets,
                combat::check_bullet_collision,
            )
                .run_if(in_state(GameState::Battle)),
        )
        .add_systems(OnExit(GameState::Battle), (combat::cleanup_battle, combat::unfreeze_camera))
        .add_systems(
            Update,
            (
                effects::update_damage_notifs,
                effects::update_screen_shake,
                effects::update_battle_ui,
                effects::update_phase_text,
                effects::update_particles,
            ),
        )
        .add_systems(Update, game_over_screen.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, victory_screen.run_if(in_state(GameState::Victory)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        ScreenShake { trauma: 0.0 },
        OverworldCamera,
    ));
}

fn setup_world(mut commands: Commands) {
    // Player in overworld
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.8, 1.0),
            custom_size: Some(Vec2::new(24.0, 24.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -220.0, 1.0)),
        Player {
            health: PLAYER_MAX_HEALTH,
            max_health: PLAYER_MAX_HEALTH,
        },
    ));

    let enemy_colors = [
        Color::srgb(1.0, 0.4, 0.4),
        Color::srgb(0.4, 1.0, 0.4),
        Color::srgb(0.4, 0.6, 1.0),
        Color::srgb(1.0, 0.9, 0.3),
        Color::srgb(1.0, 0.5, 0.8),
        Color::srgb(0.5, 1.0, 0.8),
    ];

    for i in 0..TOTAL_ROOMS {
        let y_pos = (i as f32 * ROOM_HEIGHT) - 150.0;

        // Room
        commands.spawn((
            Sprite {
                color: Color::srgb(0.12, 0.12, 0.18),
                custom_size: Some(Vec2::new(220.0, 110.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos, 0.0)),
            Room {
                index: i,
                cleared: false,
            },
        ));

        // Enemy
        commands.spawn((
            Sprite {
                color: enemy_colors[i],
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos, 0.5)),
            Enemy {
                health: 20 + (i as i32 * 5),
                max_health: 20 + (i as i32 * 5),
                room_index: i,
                attack_pattern: i,
            },
        ));

        // Boundaries
        for offset in [-56.0, 56.0] {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.4, 0.4, 0.5),
                    custom_size: Some(Vec2::new(240.0, 3.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, y_pos + offset, 0.1)),
            ));
        }
    }

    // Exit door
    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.2, 0.9),
            custom_size: Some(Vec2::new(60.0, 25.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 700.0, 0.5)),
        ExitDoor,
    ));
}

fn setup_ui(mut commands: Commands) {
    // Battle arena overlay (top screen)
    commands.spawn((
        Sprite {
            color: Color::srgba(0.08, 0.08, 0.15, 0.98),
            custom_size: Some(Vec2::new(ARENA_WIDTH + 50.0, ARENA_HEIGHT + 50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, BATTLE_ARENA_Y, 9.0)),
        Visibility::Hidden,
        BattleUI,
    ));

    // Health text - TOP LEFT
    commands.spawn((
        Text::new("♥ Player: 30/30 HP\n◆ Enemy: 20/20 HP"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        HealthText,
        Visibility::Hidden,
        BattleUI,
    ));

    // Phase text - TOP RIGHT
    commands.spawn((
        Text::new("YOUR TURN"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.3, 1.0, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(20.0),
            ..default()
        },
        PhaseText,
        Visibility::Hidden,
        BattleUI,
    ));

    // Controls - BOTTOM CENTER
    commands.spawn((
        Text::new("[SPACE] Time Attack | [2] Defend | WASD to Dodge"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        ControlsText,
        Visibility::Hidden,
        BattleUI,
    ));

    // Overworld instructions - BOTTOM CENTER
    commands.spawn((
        Text::new("WASD: Move | Approach enemies to battle | Reach the exit!"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));

    // Room counter - TOP CENTER
    commands.spawn((
        Text::new("Room 0 / 6"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.3, 1.0, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        Text::new("◆ DUNGEON GAUNTLET ◆\n\nPress [SPACE] to Start\n\nDefeat 6 enemies and reach the exit!"),
        TextFont {
            font_size: 42.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        MainMenuUI,
    ));
}

fn main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_state.set(GameState::Overworld);
    }
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn game_over_screen(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    mut enemy_query: Query<&mut Enemy>,
    mut rooms_query: Query<&mut Room>,
    mut game_progress: ResMut<GameProgress>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_progress.rooms_cleared = 0;
        if let Ok(player_entity) = player_query.single() {
            commands.entity(player_entity).insert(Transform::from_translation(Vec3::new(0.0, -220.0, 1.0)));
        }
        for mut enemy in enemy_query.iter_mut() {
            enemy.health = enemy.max_health;
        }
        for mut room in rooms_query.iter_mut() {
            room.cleared = false;
        }
        game_state.set(GameState::Overworld);
    }
}

fn victory_screen(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    mut enemy_query: Query<&mut Enemy>,
    mut rooms_query: Query<&mut Room>,
    mut game_progress: ResMut<GameProgress>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_progress.rooms_cleared = 0;
        if let Ok(player_entity) = player_query.single() {
            commands.entity(player_entity).insert(Transform::from_translation(Vec3::new(0.0, -220.0, 1.0)));
        }
        for mut enemy in enemy_query.iter_mut() {
            enemy.health = enemy.max_health;
        }
        for mut room in rooms_query.iter_mut() {
            room.cleared = false;
        }
        game_state.set(GameState::Overworld);
    }
}
