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
    room_index: usize,
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

#[derive(Component)]
struct BattleSprite;

#[derive(Component)]
struct DamageNotif {
    timer: Timer,
}

#[derive(Component)]
struct InstructionText;

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
            (battle_ui_input, update_battle_ui, update_damage_notifs)
                .run_if(in_state(GameState::Battle)),
        )
        .add_systems(OnEnter(GameState::Battle), setup_battle_sprites)
        .add_systems(OnExit(GameState::Battle), cleanup_battle_sprites)
        .add_systems(Update, game_over_restart.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, victory_restart.run_if(in_state(GameState::Victory)))
        .run();
}

fn setup_game(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);

    // Instructions
    commands.spawn((
        Text::new("WASD/Arrows: Move | [1] Attack [2] Defend | Space: Restart"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        InstructionText,
    ));

    // Player - spawn at bottom, away from first enemy
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.8, 1.0), // Light blue
            custom_size: Some(Vec2::new(16.0, 16.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -200.0, 1.0)),
        Player,
    ));

    // Create 4 vertically stacked rooms
    let room_height = 120.0;
    let enemy_colors = [
        Color::srgb(1.0, 0.3, 0.3), // Red
        Color::srgb(0.3, 1.0, 0.3), // Green
        Color::srgb(0.3, 0.3, 1.0), // Blue
        Color::srgb(1.0, 1.0, 0.3), // Yellow
    ];

    for i in 0..4 {
        let y_pos = (i as f32 * room_height) - 150.0;

        // Room background
        commands.spawn((
            Sprite {
                color: Color::srgb(0.15, 0.15, 0.2),
                custom_size: Some(Vec2::new(200.0, 100.0)),
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
                room_index: i,
            },
        ));

        // Room boundaries
        commands.spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(220.0, 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos + 51.0, 0.1)),
        ));

        commands.spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(220.0, 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos - 51.0, 0.1)),
        ));
    }

    // Exit door at the top
    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.2, 0.8), // Purple
            custom_size: Some(Vec2::new(40.0, 15.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 300.0, 0.5)),
        ExitDoor,
    ));
}

fn setup_battle_ui(mut commands: Commands) {
    // Battle background overlay
    commands.spawn((
        Sprite {
            color: Color::srgba(0.1, 0.1, 0.2, 0.95),
            custom_size: Some(Vec2::new(400.0, 250.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        Visibility::Hidden,
        BattleUI,
    ));

    // Health text
    commands.spawn((
        Text::new("Player: 20 HP | Enemy: 0 HP"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        HealthText,
        Visibility::Hidden,
        BattleUI,
    ));

    // Battle options
    commands.spawn((
        Text::new("[1] ATTACK     [2] DEFEND"),
        TextFont {
            font_size: 26.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(100.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Visibility::Hidden,
        BattleUI,
    ));
}

fn setup_battle_sprites(
    mut commands: Commands,
    battle_state: Res<CurrentBattle>,
    enemy_query: Query<&Enemy>,
) {
    // Spawn player battle sprite
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.8, 1.0),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-80.0, -20.0, 11.0)),
        BattleSprite,
    ));

    // Spawn enemy battle sprite
    if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
        let color = match enemy.room_index {
            0 => Color::srgb(1.0, 0.3, 0.3),
            1 => Color::srgb(0.3, 1.0, 0.3),
            2 => Color::srgb(0.3, 0.3, 1.0),
            _ => Color::srgb(1.0, 1.0, 0.3),
        };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(80.0, -10.0, 11.0)),
            BattleSprite,
        ));
    }
}

fn cleanup_battle_sprites(mut commands: Commands, query: Query<Entity, With<BattleSprite>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let speed = 120.0;

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

    transform.translation.x = transform.translation.x.clamp(-100.0, 100.0);
}

fn check_room_transition(
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Enemy), Without<Player>>,
    mut battle_state: ResMut<CurrentBattle>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut game_progress: ResMut<GameProgress>,
    rooms_query: Query<&mut Room>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (enemy_entity, enemy_transform, enemy) in enemy_query.iter() {
        let distance = player_transform
            .translation
            .distance(enemy_transform.translation);

        // Check if room is already cleared
        let room_cleared = rooms_query
            .iter()
            .any(|room| room.index == enemy.room_index && room.cleared);

        if distance < 35.0 && enemy.health > 0 && !room_cleared {
            battle_state.enemy_entity = enemy_entity;
            battle_state.player_health = 20;
            battle_state.player_defending = false;
            game_progress.current_room = enemy.room_index;

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
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(exit_transform) = exit_query.single() else {
        return;
    };

    let distance = player_transform
        .translation
        .distance(exit_transform.translation);

    if distance < 35.0 && game_progress.rooms_cleared >= game_progress.total_rooms {
        game_state.set(GameState::Victory);
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn battle_ui_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut battle_state: ResMut<CurrentBattle>,
    mut enemy_query: Query<&mut Enemy>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut game_progress: ResMut<GameProgress>,
    mut rooms_query: Query<&mut Room>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        // Attack
        if let Ok(mut enemy) = enemy_query.get_mut(battle_state.enemy_entity) {
            let damage = if battle_state.player_defending { 3 } else { 5 };
            enemy.health -= damage;

            // Spawn damage notification
            spawn_damage_notif(&mut commands, format!("-{} HP", damage), Vec3::new(80.0, 30.0, 12.0), Color::srgb(1.0, 0.3, 0.3));

            // Enemy attacks back
            let enemy_damage = 2;
            battle_state.player_health -= enemy_damage;
            spawn_damage_notif(&mut commands, format!("-{} HP", enemy_damage), Vec3::new(-80.0, 10.0, 12.0), Color::srgb(1.0, 0.8, 0.3));

            battle_state.player_defending = false;
        }
    }

    if keyboard_input.just_pressed(KeyCode::Digit2) {
        // Defend
        battle_state.player_defending = true;
        spawn_damage_notif(&mut commands, "DEFENDING".to_string(), Vec3::new(-80.0, 10.0, 12.0), Color::srgb(0.3, 1.0, 0.3));

        // Enemy attacks (reduced damage)
        battle_state.player_health -= 1;
        spawn_damage_notif(&mut commands, "-1 HP".to_string(), Vec3::new(-80.0, -10.0, 12.0), Color::srgb(1.0, 0.8, 0.3));
    }

    // Check battle outcome
    if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
        if battle_state.player_health <= 0 {
            game_state.set(GameState::GameOver);
            for mut visibility in battle_ui.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        } else if enemy.health <= 0 {
            // Mark room as cleared
            for mut room in rooms_query.iter_mut() {
                if room.index == game_progress.current_room {
                    room.cleared = true;
                }
            }
            
            game_progress.rooms_cleared += 1;
            game_state.set(GameState::Overworld);
            
            for mut visibility in battle_ui.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn spawn_damage_notif(commands: &mut Commands, text: String, position: Vec3, color: Color) {
    commands.spawn((
        Text::new(text),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
        DamageNotif {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
    ));
}

fn update_damage_notifs(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageNotif, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut notif, mut transform) in query.iter_mut() {
        notif.timer.tick(time.delta());
        
        // Float upward
        transform.translation.y += 30.0 * time.delta_secs();

        if notif.timer.is_finished() {
            commands.entity(entity).despawn();
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
                "Player: {} HP  |  Enemy: {} HP",
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
    mut enemy_query: Query<&mut Enemy>,
    mut rooms_query: Query<&mut Room>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_progress.rooms_cleared = 0;
        game_progress.current_room = 0;

        for mut visibility in battle_ui.iter_mut() {
            *visibility = Visibility::Hidden;
        }

        if let Ok(player_entity) = player_query.single() {
            commands
                .entity(player_entity)
                .insert(Transform::from_translation(Vec3::new(0.0, -200.0, 1.0)));
        }

        // Reset enemies
        for mut enemy in enemy_query.iter_mut() {
            enemy.health = enemy.max_health;
        }

        // Reset rooms
        for mut room in rooms_query.iter_mut() {
            room.cleared = false;
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
    mut rooms_query: Query<&mut Room>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_progress.rooms_cleared = 0;
        game_progress.current_room = 0;

        for mut visibility in battle_ui.iter_mut() {
            *visibility = Visibility::Hidden;
        }

        if let Ok(player_entity) = player_query.single() {
            commands
                .entity(player_entity)
                .insert(Transform::from_translation(Vec3::new(0.0, -200.0, 1.0)));
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
