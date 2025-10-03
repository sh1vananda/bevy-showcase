// main.rs
use bevy::prelude::*;

// Game states
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Overworld,
    Battle,
    GameOver,
    Victory,
}

// Components
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy {
    health: i32,
    max_health: i32,
}

#[derive(Component)]
struct Room {
    index: usize,
    cleared: bool,
}

#[derive(Component)]
struct ExitDoor;

#[derive(Component)]
struct BattleUI;

#[derive(Component)]
struct HealthText;

// Resources
#[derive(Resource)]
struct CurrentBattle {
    enemy_entity: Entity,
    player_health: i32,
    player_defending: bool,
}

#[derive(Resource)]
struct GameProgress {
    current_room: usize,
    rooms_cleared: usize,
    total_rooms: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .insert_resource(CurrentBattle {
            enemy_entity: Entity::PLACEHOLDER,
            player_health: 20,
            player_defending: false,
        })
        .insert_resource(GameProgress {
            current_room: 0,
            rooms_cleared: 0,
            total_rooms: 4,
        })
        .add_systems(Startup, (setup_game, setup_battle_ui))
        .add_systems(
            Update,
            (
                player_movement.run_if(in_state(GameState::Overworld)),
                check_room_transition.run_if(in_state(GameState::Overworld)),
                check_exit_door.run_if(in_state(GameState::Overworld)),
                camera_follow.run_if(in_state(GameState::Overworld)),
            ),
        )
        .add_systems(
            Update,
            (battle_ui_input, update_battle_ui).run_if(in_state(GameState::Battle)),
        )
        .add_systems(Update, game_over_restart.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, victory_restart.run_if(in_state(GameState::Victory)))
        .run();
}

fn setup_game(mut commands: Commands) {
    // Camera - Camera2dBundle is deprecated, use Camera2d component
    commands.spawn(Camera2d);

    // Player
    commands.spawn((
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(16.0, 16.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -150.0, 1.0)),
        Player,
    ));

    // Create 4 vertically stacked rooms
    let room_height = 100.0;
    let enemy_colors = [
        Color::srgb(1.0, 0.0, 0.0),  // RED
        Color::srgb(0.0, 1.0, 0.0),  // GREEN
        Color::srgb(0.0, 0.0, 1.0),  // BLUE
        Color::srgb(1.0, 1.0, 0.0),  // YELLOW
    ];

    for i in 0..4 {
        let y_pos = (i as f32 * room_height) - 150.0;
        
        // Room background
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.2, 0.2),
                custom_size: Some(Vec2::new(200.0, 80.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos, 0.0)),
            Room {
                index: i,
                cleared: false,
            },
        ));

        // Enemy in each room
        commands.spawn((
            Sprite {
                color: enemy_colors[i],
                custom_size: Some(Vec2::new(24.0, 24.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos, 0.5)),
            Enemy {
                health: 10 + (i as i32 * 5),
                max_health: 10 + (i as i32 * 5),
            },
        ));

        // Room boundaries (top and bottom)
        commands.spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(220.0, 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos + 41.0, 0.1)),
        ));

        commands.spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(220.0, 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos - 41.0, 0.1)),
        ));
    }

    // Exit door at the top
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.0, 0.5),  // PURPLE
            custom_size: Some(Vec2::new(30.0, 10.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 250.0, 0.5)),
        ExitDoor,
    ));
}

fn setup_battle_ui(mut commands: Commands) {
    // Battle background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.3),
            custom_size: Some(Vec2::new(300.0, 200.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
        Visibility::Hidden,
        BattleUI,
    ));

    // Health text
    commands.spawn((
        Text::new("Player Health: 20\nEnemy Health: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            ..default()
        },
        HealthText,
        BattleUI,
    ));

    // Battle options
    let options = ["[1] Attack", "[2] Defend"];
    for (i, option) in options.iter().enumerate() {
        commands.spawn((
            Text::new(*option),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-30.0),
                left: Val::Px(-50.0 + (i as f32 * 150.0)),
                ..default()
            },
            BattleUI,
        ));
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = query.single_mut() else { return };
    let speed = 100.0;

    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= speed * time.delta_secs();
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        transform.translation.x += speed * time.delta_secs();
    }
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        transform.translation.y += speed * time.delta_secs();
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= speed * time.delta_secs();
    }

    // Simple bounds checking
    transform.translation.x = transform.translation.x.clamp(-100.0, 100.0);
}

fn check_room_transition(
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Enemy), Without<Player>>,
    mut battle_state: ResMut<CurrentBattle>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut game_progress: ResMut<GameProgress>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    
    for (enemy_entity, enemy_transform, enemy) in enemy_query.iter() {
        let distance = player_transform.translation.distance(enemy_transform.translation);
        
        if distance < 30.0 && enemy.health > 0 {
            battle_state.enemy_entity = enemy_entity;
            battle_state.player_health = 20;
            battle_state.player_defending = false;
            game_progress.current_room = 0;
            
            for mut visibility in battle_ui.iter_mut() {
                *visibility = Visibility::Visible;
            }
            
            game_state.set(GameState::Battle);
            break;
        }
    }
}

fn check_exit_door(
    player_query: Query<&Transform, With<Player>>,
    exit_query: Query<&Transform, (With<ExitDoor>, Without<Player>)>,
    game_progress: Res<GameProgress>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(exit_transform) = exit_query.single() else { return };
    
    let distance = player_transform.translation.distance(exit_transform.translation);
    
    if distance < 30.0 && game_progress.rooms_cleared >= game_progress.total_rooms {
        game_state.set(GameState::Victory);
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return };

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn battle_ui_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut battle_state: ResMut<CurrentBattle>,
    mut enemy_query: Query<&mut Enemy>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut game_progress: ResMut<GameProgress>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        if let Ok(mut enemy) = enemy_query.get_mut(battle_state.enemy_entity) {
            let damage = if battle_state.player_defending { 3 } else { 5 };
            enemy.health -= damage;
            
            battle_state.player_health -= 2;
            battle_state.player_defending = false;
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        battle_state.player_defending = true;
        battle_state.player_health -= 1;
    }
    
    if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
        if battle_state.player_health <= 0 {
            game_state.set(GameState::GameOver);
            for mut visibility in battle_ui.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        } else if enemy.health <= 0 {
            game_progress.rooms_cleared += 1;
            game_state.set(GameState::Overworld);
            for mut visibility in battle_ui.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn update_battle_ui(
    battle_state: Res<CurrentBattle>,
    enemy_query: Query<&Enemy>,
    mut text_query: Query<&mut Text, With<HealthText>>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
            **text = format!(
                "Player Health: {}\nEnemy Health: {}",
                battle_state.player_health.max(0),
                enemy.health.max(0)
            );
        }
    }
}

fn game_over_restart(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut game_progress: ResMut<GameProgress>,
    player_query: Query<Entity, With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_progress.rooms_cleared = 0;
        game_progress.current_room = 0;
        
        for mut visibility in battle_ui.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        
        if let Ok(player_entity) = player_query.single() {
            commands.entity(player_entity).insert(Transform::from_translation(Vec3::new(0.0, -150.0, 1.0)));
        }
        
        game_state.set(GameState::Overworld);
    }
}

fn victory_restart(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut game_progress: ResMut<GameProgress>,
    player_query: Query<Entity, With<Player>>,
    mut enemy_query: Query<&mut Enemy>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_progress.rooms_cleared = 0;
        game_progress.current_room = 0;
        
        for mut visibility in battle_ui.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        
        if let Ok(player_entity) = player_query.single() {
            commands.entity(player_entity).insert(Transform::from_translation(Vec3::new(0.0, -150.0, 1.0)));
        }
        
        for mut enemy in enemy_query.iter_mut() {
            enemy.health = enemy.max_health;
        }
        
        game_state.set(GameState::Overworld);
    }
}
