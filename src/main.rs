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

// Battle phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BattlePhase {
    PlayerTurn,
    EnemyTelegraph,
    EnemyAttack,
    Resolution,
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
struct PlayerSprite;

#[derive(Component)]
struct EnemySprite;

#[derive(Component)]
struct DamageNotif {
    timer: Timer,
    velocity: Vec2,
}

#[derive(Component)]
struct InstructionText;

#[derive(Component)]
struct TelegraphIndicator;

#[derive(Component)]
struct PhaseText;

#[derive(Component)]
struct ScreenShake {
    trauma: f32,
}

#[derive(Component)]
struct AttackFlash {
    timer: Timer,
    intensity: f32,
}

// Resources
#[derive(Resource)]
struct CurrentBattle {
    enemy_entity: Entity,
    player_health: i32,
    max_player_health: i32,
    player_defended: bool,
    phase: BattlePhase,
    phase_timer: Timer,
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
            max_player_health: 20,
            player_defended: false,
            phase: BattlePhase::PlayerTurn,
            phase_timer: Timer::from_seconds(0.0, TimerMode::Once),
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
                player_movement,
                check_room_transition,
                check_exit_door,
                camera_follow,
                update_screen_shake,
            )
                .run_if(in_state(GameState::Overworld)),
        )
        .add_systems(
            Update,
            (
                battle_system,
                update_battle_ui,
                update_damage_notifs,
                update_attack_flash,
                update_telegraph,
            )
                .run_if(in_state(GameState::Battle)),
        )
        .add_systems(OnEnter(GameState::Battle), setup_battle_sprites)
        .add_systems(OnExit(GameState::Battle), cleanup_battle_sprites)
        .add_systems(Update, game_over_restart.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, victory_restart.run_if(in_state(GameState::Victory)))
        .run();
}

fn setup_game(mut commands: Commands) {
    // Camera with shake
    commands.spawn((
        Camera2d,
        ScreenShake { trauma: 0.0 },
    ));

    // Instructions - top center
    commands.spawn((
        Text::new("WASD: Move | Battle: [1] Attack [2] Defend | Space: Restart"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        InstructionText,
    ));

    // Progress tracker - top left
    commands.spawn((
        Text::new("Rooms Cleared: 0/4"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.3, 1.0, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));

    // Player
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.8, 1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -220.0, 1.0)),
        Player,
    ));

    // Create 4 vertically stacked rooms
    let room_height = 130.0;
    let enemy_colors = [
        Color::srgb(1.0, 0.3, 0.3),
        Color::srgb(0.3, 1.0, 0.3),
        Color::srgb(0.3, 0.6, 1.0),
        Color::srgb(1.0, 0.9, 0.2),
    ];

    for i in 0..4 {
        let y_pos = (i as f32 * room_height) - 150.0;

        // Room background
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
                custom_size: Some(Vec2::new(28.0, 28.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_pos, 0.5)),
            Enemy {
                health: 15 + (i as i32 * 5),
                max_health: 15 + (i as i32 * 5),
                room_index: i,
            },
        ));

        // Room boundaries
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
            custom_size: Some(Vec2::new(50.0, 20.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 320.0, 0.5)),
        ExitDoor,
    ));
}

fn setup_battle_ui(mut commands: Commands) {
    // Dark overlay
    commands.spawn((
        Sprite {
            color: Color::srgba(0.05, 0.05, 0.15, 0.97),
            custom_size: Some(Vec2::new(500.0, 300.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        Visibility::Hidden,
        BattleUI,
    ));

    // Health bar background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.2),
            custom_size: Some(Vec2::new(300.0, 40.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 100.0, 11.0)),
        Visibility::Hidden,
        BattleUI,
    ));

    // Health text - centered
    commands.spawn((
        Text::new("Player: 20/20 HP | Enemy: 15/15 HP"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(150.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        HealthText,
        Visibility::Hidden,
        BattleUI,
    ));

    // Phase indicator - top center
    commands.spawn((
        Text::new("YOUR TURN"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(0.3, 1.0, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(80.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        PhaseText,
        Visibility::Hidden,
        BattleUI,
    ));

    // Battle options - bottom center
    commands.spawn((
        Text::new("[1] ATTACK  |  [2] DEFEND"),
        TextFont {
            font_size: 26.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.85, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(80.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
        BattleUI,
    ));
}

fn setup_battle_sprites(mut commands: Commands, battle_state: Res<CurrentBattle>, enemy_query: Query<&Enemy>) {
    // Player sprite
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.8, 1.0),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-100.0, -30.0, 11.0)),
        BattleSprite,
        PlayerSprite,
    ));

    // Enemy sprite
    if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
        let color = match enemy.room_index {
            0 => Color::srgb(1.0, 0.3, 0.3),
            1 => Color::srgb(0.3, 1.0, 0.3),
            2 => Color::srgb(0.3, 0.6, 1.0),
            _ => Color::srgb(1.0, 0.9, 0.2),
        };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(100.0, -20.0, 11.0)),
            BattleSprite,
            EnemySprite,
        ));
    }
}

fn cleanup_battle_sprites(mut commands: Commands, query: Query<Entity, With<BattleSprite>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn battle_system(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut battle_state: ResMut<CurrentBattle>,
    mut enemy_query: Query<&mut Enemy>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut phase_text: Query<(&mut Text, &mut TextColor), With<PhaseText>>,
    mut game_progress: ResMut<GameProgress>,
    mut rooms_query: Query<&mut Room>,
    mut shake_query: Query<&mut ScreenShake>,
    enemy_sprite_query: Query<(Entity, &Transform), With<EnemySprite>>,
    player_sprite_query: Query<Entity, With<PlayerSprite>>,
) {
    battle_state.phase_timer.tick(time.delta());

    match battle_state.phase {
        BattlePhase::PlayerTurn => {
            // Update phase text
            if let Ok((mut text, mut color)) = phase_text.single_mut() {
                **text = "YOUR TURN".to_string();
                color.0 = Color::srgb(0.3, 1.0, 0.3);
            }

            if keyboard_input.just_pressed(KeyCode::Digit1) {
                // Attack
                if let Ok(mut enemy) = enemy_query.get_mut(battle_state.enemy_entity) {
                    let damage = 6;
                    enemy.health -= damage;

                    // Visual feedback
                    spawn_damage_notif(&mut commands, format!("-{}", damage), Vec3::new(100.0, 30.0, 12.0), Color::srgb(1.0, 0.3, 0.3));
                    add_screen_shake(&mut shake_query, 0.3);
                    
                    // Flash enemy
                    if let Ok((enemy_entity, _)) = enemy_sprite_query.single() {
                        commands.entity(enemy_entity).insert(AttackFlash {
                            timer: Timer::from_seconds(0.15, TimerMode::Once),
                            intensity: 1.0,
                        });
                    }

                    battle_state.player_defended = false;
                    battle_state.phase = BattlePhase::Resolution;
                    battle_state.phase_timer = Timer::from_seconds(0.8, TimerMode::Once);
                }
            } else if keyboard_input.just_pressed(KeyCode::Digit2) {
                // Defend
                battle_state.player_defended = true;
                spawn_damage_notif(&mut commands, "DEFENDING!".to_string(), Vec3::new(-100.0, 20.0, 12.0), Color::srgb(0.3, 1.0, 1.0));
                
                battle_state.phase = BattlePhase::Resolution;
                battle_state.phase_timer = Timer::from_seconds(0.8, TimerMode::Once);
            }
        }
        BattlePhase::Resolution => {
            if let Ok((mut text, mut color)) = phase_text.single_mut() {
                **text = "".to_string();
                color.0 = Color::WHITE;
            }

            if battle_state.phase_timer.just_finished() {
                // Check if battle ended
                if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
                    if battle_state.player_health <= 0 {
                        game_state.set(GameState::GameOver);
                        for mut visibility in battle_ui.iter_mut() {
                            *visibility = Visibility::Hidden;
                        }
                        return;
                    } else if enemy.health <= 0 {
                        // Victory
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
                        return;
                    }
                }

                // Continue to enemy turn
                battle_state.phase = BattlePhase::EnemyTelegraph;
                battle_state.phase_timer = Timer::from_seconds(1.5, TimerMode::Once);
                
                // Spawn telegraph indicator
                if let Ok((_, transform)) = enemy_sprite_query.single() {
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(1.0, 0.2, 0.2, 0.6),
                            custom_size: Some(Vec2::new(60.0, 60.0)),
                            ..default()
                        },
                        Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, 0.5)),
                        TelegraphIndicator,
                        BattleSprite,
                    ));
                }
            }
        }
        BattlePhase::EnemyTelegraph => {
            if let Ok((mut text, mut color)) = phase_text.single_mut() {
                **text = "ENEMY ATTACKING!".to_string();
                color.0 = Color::srgb(1.0, 0.3, 0.3);
            }

            // Can still defend during telegraph
            if keyboard_input.just_pressed(KeyCode::Digit2) && !battle_state.player_defended {
                battle_state.player_defended = true;
                spawn_damage_notif(&mut commands, "BLOCKED!".to_string(), Vec3::new(-100.0, 20.0, 12.0), Color::srgb(0.3, 1.0, 1.0));
            }

            if battle_state.phase_timer.just_finished() {
                battle_state.phase = BattlePhase::EnemyAttack;
                battle_state.phase_timer = Timer::from_seconds(0.3, TimerMode::Once);
            }
        }
        BattlePhase::EnemyAttack => {
            if battle_state.phase_timer.just_finished() {
                let damage = if battle_state.player_defended { 1 } else { 4 };
                battle_state.player_health -= damage;

                spawn_damage_notif(&mut commands, format!("-{}", damage), Vec3::new(-100.0, 0.0, 12.0), Color::srgb(1.0, 0.7, 0.3));
                add_screen_shake(&mut shake_query, if battle_state.player_defended { 0.1 } else { 0.4 });
                
                // Flash player
                if let Ok(player_entity) = player_sprite_query.single() {
                    commands.entity(player_entity).insert(AttackFlash {
                        timer: Timer::from_seconds(0.15, TimerMode::Once),
                        intensity: 1.0,
                    });
                }

                battle_state.player_defended = false;
                battle_state.phase = BattlePhase::PlayerTurn;
            }
        }
    }
}

fn update_telegraph(
    time: Res<Time>,
    mut query: Query<&mut Sprite, With<TelegraphIndicator>>,
) {
    for mut sprite in query.iter_mut() {
        let pulse = (time.elapsed_secs() * 6.0).sin() * 0.5 + 0.5;
        sprite.color.set_alpha(0.3 + pulse * 0.4);
    }
}

fn add_screen_shake(shake_query: &mut Query<&mut ScreenShake>, amount: f32) {
    if let Ok(mut shake) = shake_query.single_mut() {
        shake.trauma = (shake.trauma + amount).min(1.0);
    }
}

fn update_screen_shake(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ScreenShake), With<Camera2d>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    if let Ok((mut camera_transform, mut shake)) = query.single_mut() {
        shake.trauma = (shake.trauma - time.delta_secs() * 1.5).max(0.0);
        
        let shake_amount = shake.trauma * shake.trauma;
        let offset_x = (time.elapsed_secs() * 20.0).sin() * shake_amount * 8.0;
        let offset_y = (time.elapsed_secs() * 25.0).cos() * shake_amount * 8.0;
        
        if let Ok(player_transform) = player_query.single() {
            camera_transform.translation.x = player_transform.translation.x + offset_x;
            camera_transform.translation.y = player_transform.translation.y + offset_y;
        }
    }
}

fn update_attack_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut AttackFlash)>,
) {
    for (entity, mut sprite, mut flash) in query.iter_mut() {
        flash.timer.tick(time.delta());
        
        let t = flash.timer.fraction();
        let flash_color = Color::WHITE.mix(&sprite.color, t);
        sprite.color = flash_color;
        
        if flash.timer.finished() {
            commands.entity(entity).remove::<AttackFlash>();
        }
    }
}

fn spawn_damage_notif(commands: &mut Commands, text: String, position: Vec3, color: Color) {
    commands.spawn((
        Text::new(text),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
        DamageNotif {
            timer: Timer::from_seconds(1.2, TimerMode::Once),
            velocity: Vec2::new((rand::random::<f32>() - 0.5) * 20.0, 40.0),
        },
        BattleSprite,
    ));
}

fn update_damage_notifs(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageNotif, &mut Transform, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut notif, mut transform, mut color) in query.iter_mut() {
        notif.timer.tick(time.delta());
        
        transform.translation.x += notif.velocity.x * time.delta_secs();
        transform.translation.y += notif.velocity.y * time.delta_secs();
        
        let alpha = 1.0 - notif.timer.fraction();
        color.0.set_alpha(alpha);

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
                "Player: {}/{} HP  |  Enemy: {}/{} HP",
                battle_state.player_health.max(0),
                battle_state.max_player_health,
                enemy.health.max(0),
                enemy.max_health
            );
        }
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = query.single_mut() else { return };
    let speed = 130.0;

    let mut direction = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        transform.translation.x += direction.x * speed * time.delta_secs();
        transform.translation.y += direction.y * speed * time.delta_secs();
    }

    transform.translation.x = transform.translation.x.clamp(-110.0, 110.0);
}

fn check_room_transition(
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Enemy), Without<Player>>,
    mut battle_state: ResMut<CurrentBattle>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut game_progress: ResMut<GameProgress>,
    rooms_query: Query<&Room>,
) {
    let Ok(player_transform) = player_query.single() else { return };

    for (enemy_entity, enemy_transform, enemy) in enemy_query.iter() {
        let distance = player_transform.translation.distance(enemy_transform.translation);

        let room_cleared = rooms_query
            .iter()
            .any(|room| room.index == enemy.room_index && room.cleared);

        if distance < 40.0 && enemy.health > 0 && !room_cleared {
            battle_state.enemy_entity = enemy_entity;
            battle_state.player_health = 20;
            battle_state.player_defended = false;
            battle_state.phase = BattlePhase::PlayerTurn;
            battle_state.phase_timer = Timer::from_seconds(0.0, TimerMode::Once);
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
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(exit_transform) = exit_query.single() else { return };

    let distance = player_transform.translation.distance(exit_transform.translation);

    if distance < 40.0 && game_progress.rooms_cleared >= game_progress.total_rooms {
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

        for mut visibility in battle_ui.iter_mut() {
            *visibility = Visibility::Hidden;
        }

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

        for mut visibility in battle_ui.iter_mut() {
            *visibility = Visibility::Hidden;
        }

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