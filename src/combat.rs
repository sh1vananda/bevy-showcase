use bevy::prelude::*;
use crate::components::*;
use rand::Rng;

pub fn freeze_camera(mut camera_query: Query<&mut Transform, With<OverworldCamera>>) {
    if let Ok(mut transform) = camera_query.single_mut() {
        transform.translation = Vec3::new(0.0, BATTLE_ARENA_Y, transform.translation.z);
    }
}

pub fn unfreeze_camera() {}

pub fn setup_battle(
    mut commands: Commands,
    battle_state: Res<CurrentBattle>,
    enemy_query: Query<&Enemy>,
    mut spawner: ResMut<BulletSpawner>,
) {
    spawner.timer = Timer::from_seconds(0.5, TimerMode::Repeating);
    
    // Arena border
    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.3, 0.3),
            custom_size: Some(Vec2::new(ARENA_WIDTH + 6.0, ARENA_HEIGHT + 6.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, BATTLE_ARENA_Y, 10.0)),
        BattleSprite,
    ));

    // Arena interior
    commands.spawn((
        Sprite {
            color: Color::srgb(0.05, 0.05, 0.08),
            custom_size: Some(Vec2::new(ARENA_WIDTH, ARENA_HEIGHT)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, BATTLE_ARENA_Y, 10.1)),
        BattleSprite,
    ));

    // Player soul
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.2, 0.2),
            custom_size: Some(Vec2::new(22.0, 22.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, BATTLE_ARENA_Y - 50.0, 11.0)),
        Player {
            health: PLAYER_MAX_HEALTH,
            max_health: PLAYER_MAX_HEALTH,
        },
        BattleSprite,
        PlayerSprite,
    ));

    // Enemy sprite
    if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
        let size = 50.0 + (enemy.attack_pattern as f32 * 7.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.9, 0.45, 0.35),
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, BATTLE_ARENA_Y + 80.0, 11.0)),
            BattleSprite,
            EnemySprite,
        ));
    }

    // Timing bar background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.25, 0.25, 0.25),
            custom_size: Some(Vec2::new(320.0, 24.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, BATTLE_ARENA_Y - 110.0, 11.0)),
        BattleSprite,
    ));

    // Perfect zone
    commands.spawn((
        Sprite {
            color: Color::srgba(0.3, 1.0, 0.3, 0.4),
            custom_size: Some(Vec2::new(50.0, 24.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, BATTLE_ARENA_Y - 110.0, 11.1)),
        BattleSprite,
    ));

    // Attack indicator
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 0.3),
            custom_size: Some(Vec2::new(10.0, 32.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-160.0, BATTLE_ARENA_Y - 110.0, 11.5)),
        AttackIndicator {
            speed: 220.0,
            direction: 1.0,
        },
        BattleSprite,
    ));
}

pub fn cleanup_battle(
    mut commands: Commands,
    battle_sprites: Query<Entity, With<BattleSprite>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
) {
    for mut visibility in battle_ui.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    for entity in battle_sprites.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn battle_phase_system(
    time: Res<Time>,
    mut battle_state: ResMut<CurrentBattle>,
    mut game_state: ResMut<NextState<GameState>>,
    player_query: Query<&Player, With<PlayerSprite>>,
    enemy_query: Query<&Enemy>,
    mut rooms_query: Query<&mut Room>,
    mut game_progress: ResMut<GameProgress>,
    mut commands: Commands,
    bullets: Query<Entity, With<Bullet>>,
) {
    battle_state.phase_timer.tick(time.delta());

    match battle_state.phase {
        BattlePhase::Intro => {
            if battle_state.phase_timer.just_finished() {
                battle_state.phase = BattlePhase::PlayerTurn;
            }
        }
        BattlePhase::PlayerTurn => {}
        BattlePhase::EnemyTelegraph => {
            if battle_state.phase_timer.just_finished() {
                battle_state.phase = BattlePhase::BulletHell;
                battle_state.phase_timer = Timer::from_seconds(4.0, TimerMode::Once);
            }
        }
        BattlePhase::BulletHell => {
            if battle_state.phase_timer.just_finished() {
                battle_state.phase = BattlePhase::Resolution;
                battle_state.phase_timer = Timer::from_seconds(1.0, TimerMode::Once);
                
                for bullet in bullets.iter() {
                    commands.entity(bullet).despawn();
                }
            }
        }
        BattlePhase::Resolution => {
            if battle_state.phase_timer.just_finished() {
                if let Ok(player) = player_query.single() {
                    if player.health <= 0 {
                        game_state.set(GameState::GameOver);
                        return;
                    }
                }

                if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
                    if enemy.health <= 0 {
                        for mut room in rooms_query.iter_mut() {
                            if room.index == game_progress.current_room {
                                room.cleared = true;
                            }
                        }
                        game_progress.rooms_cleared += 1;
                        game_state.set(GameState::Overworld);
                        return;
                    }
                }

                battle_state.phase = BattlePhase::PlayerTurn;
                battle_state.combo_count = 0;
                battle_state.player_defended = false;
            }
        }
    }
}

pub fn update_attack_indicator(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut AttackIndicator)>,
    battle_state: Res<CurrentBattle>,
) {
    if battle_state.phase != BattlePhase::PlayerTurn {
        return;
    }

    for (mut transform, mut indicator) in query.iter_mut() {
        transform.translation.x += indicator.speed * indicator.direction * time.delta_secs();

        if transform.translation.x > 160.0 {
            transform.translation.x = 160.0;
            indicator.direction = -1.0;
        } else if transform.translation.x < -160.0 {
            transform.translation.x = -160.0;
            indicator.direction = 1.0;
        }
    }
}

pub fn player_turn_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut battle_state: ResMut<CurrentBattle>,
    mut commands: Commands,
    indicator_query: Query<&Transform, With<AttackIndicator>>,
    enemy_query: Query<&Transform, With<EnemySprite>>,
    mut enemy_data: Query<&mut Enemy>,
    mut shake_query: Query<&mut ScreenShake>,
) {
    if battle_state.phase != BattlePhase::PlayerTurn {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        if let Ok(indicator_transform) = indicator_query.single() {
            let indicator_x = indicator_transform.translation.x;
            let distance = indicator_x.abs();
            
            let (damage, text, color) = if distance < 25.0 {
                (15, "★ PERFECT! ★", Color::srgb(1.0, 1.0, 0.3))
            } else if distance < 70.0 {
                (10, "GOOD!", Color::srgb(0.3, 1.0, 0.3))
            } else {
                (5, "Hit", Color::srgb(0.7, 0.7, 0.7))
            };

            if let Ok(mut enemy) = enemy_data.get_mut(battle_state.enemy_entity) {
                enemy.health -= damage;

                if let Ok(enemy_transform) = enemy_query.single() {
                    spawn_damage(&mut commands, format!("{}\n-{}", text, damage), 
                        Vec3::new(80.0, BATTLE_ARENA_Y + 80.0, 15.0), color);
                    spawn_particles(&mut commands, enemy_transform.translation, color, 12);

                    if let Ok(mut shake) = shake_query.single_mut() {
                        shake.trauma = if distance < 25.0 { 0.6 } else { 0.3 };
                    }
                }
            }

            start_enemy_turn(&mut battle_state, &mut commands, &enemy_query);
        }
    }

    if keyboard.just_pressed(KeyCode::Digit2) {
        battle_state.player_defended = true;
        spawn_text(&mut commands, "⚔ DEFENDING ⚔", Vec3::new(0.0, BATTLE_ARENA_Y + 10.0, 15.0), Color::srgb(0.3, 0.8, 1.0));
        start_enemy_turn(&mut battle_state, &mut commands, &enemy_query);
    }
}

fn start_enemy_turn(
    battle_state: &mut ResMut<CurrentBattle>,
    commands: &mut Commands,
    enemy_query: &Query<&Transform, With<EnemySprite>>,
) {
    battle_state.phase = BattlePhase::EnemyTelegraph;
    battle_state.phase_timer = Timer::from_seconds(1.5, TimerMode::Once);

    if let Ok(transform) = enemy_query.single() {
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 0.3, 0.3, 0.7),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            Transform::from_translation(transform.translation),
            Telegraph {
                timer: Timer::from_seconds(1.5, TimerMode::Once),
            },
            BattleSprite,
        ));
    }
}

pub fn spawn_bullet_patterns(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<BulletSpawner>,
    battle_state: Res<CurrentBattle>,
    enemy_query: Query<&Enemy>,
    enemy_sprite_query: Query<&Transform, With<EnemySprite>>,
) {
    if battle_state.phase != BattlePhase::BulletHell {
        return;
    }

    spawner.timer.tick(time.delta());

    if spawner.timer.just_finished() {
        if let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) {
            if let Ok(transform) = enemy_sprite_query.single() {
                match enemy.attack_pattern % 4 {
                    0 => spawn_wave(&mut commands, transform.translation),
                    1 => spawn_spiral(&mut commands, transform.translation),
                    2 => spawn_spread(&mut commands, transform.translation),
                    _ => spawn_cross(&mut commands, transform.translation),
                }
            }
        }
    }
}

fn spawn_wave(commands: &mut Commands, origin: Vec3) {
    for i in 0..3 {
        let offset_x = (i as f32 - 1.0) * 60.0;
        spawn_bullet(commands, origin + Vec3::new(offset_x, 0.0, 0.0), Vec2::new(0.0, -70.0));
    }
}

fn spawn_spiral(commands: &mut Commands, origin: Vec3) {
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        let vel = Vec2::new(angle.cos() * 65.0, angle.sin() * 65.0);
        spawn_bullet(commands, origin, vel);
    }
}

fn spawn_spread(commands: &mut Commands, origin: Vec3) {
    for i in 0..5 {
        let angle = -0.6 + (i as f32 * 0.3);
        let vel = Vec2::new(angle.sin() * 75.0, -angle.cos() * 75.0);
        spawn_bullet(commands, origin, vel);
    }
}

fn spawn_cross(commands: &mut Commands, origin: Vec3) {
    let dirs = [Vec2::new(1.0, 0.0), Vec2::new(-1.0, 0.0), Vec2::new(0.0, 1.0), Vec2::new(0.0, -1.0)];
    for dir in dirs {
        spawn_bullet(commands, origin, dir * 70.0);
    }
}

fn spawn_bullet(commands: &mut Commands, position: Vec3, velocity: Vec2) {
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.95, 0.2),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_translation(position),
        Bullet {
            velocity,
            damage: 4,
            lifetime: Timer::from_seconds(8.0, TimerMode::Once),
        },
        BattleSprite,
    ));
}

pub fn update_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Bullet)>,
) {
    for (entity, mut transform, mut bullet) in query.iter_mut() {
        bullet.lifetime.tick(time.delta());
        
        transform.translation.x += bullet.velocity.x * time.delta_secs();
        transform.translation.y += bullet.velocity.y * time.delta_secs();

        if bullet.lifetime.is_finished() || 
           transform.translation.x.abs() > 500.0 || 
           (transform.translation.y - BATTLE_ARENA_Y).abs() > 300.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn check_bullet_collision(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &Bullet)>,
    mut player_query: Query<(&Transform, &mut Player), With<PlayerSprite>>,
    battle_state: Res<CurrentBattle>,
) {
    let Ok((player_transform, mut player)) = player_query.single_mut() else { return };

    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        let distance = player_transform.translation.distance(bullet_transform.translation);

        if distance < 25.0 {
            let damage = if battle_state.player_defended { 1 } else { bullet.damage };
            player.health -= damage;

            spawn_damage(&mut commands, format!("-{}", damage), 
                Vec3::new(-100.0, BATTLE_ARENA_Y - 50.0, 15.0), Color::srgb(1.0, 0.6, 0.3));
            spawn_particles(&mut commands, bullet_transform.translation, Color::srgb(1.0, 0.7, 0.3), 10);

            commands.entity(bullet_entity).despawn();
        }
    }
}

pub fn bullet_hell_player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<PlayerSprite>>,
    time: Res<Time>,
    battle_state: Res<CurrentBattle>,
) {
    if battle_state.phase != BattlePhase::BulletHell {
        return;
    }

    let Ok(mut transform) = query.single_mut() else { return };
    let speed = 150.0;
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
    }

    transform.translation.x += direction.x * speed * time.delta_secs();
    transform.translation.y += direction.y * speed * time.delta_secs();

    let half_w = ARENA_WIDTH / 2.0 - 15.0;
    let half_h = ARENA_HEIGHT / 2.0 - 15.0;
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
    transform.translation.y = transform.translation.y.clamp(BATTLE_ARENA_Y - half_h, BATTLE_ARENA_Y + half_h);
}

pub fn update_telegraph(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut Telegraph)>,
) {
    for (entity, mut sprite, mut telegraph) in query.iter_mut() {
        telegraph.timer.tick(time.delta());
        
        let pulse = (time.elapsed_secs() * 15.0).sin() * 0.5 + 0.5;
        sprite.color.set_alpha(0.5 + pulse * 0.4);

        if telegraph.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_damage(commands: &mut Commands, text: String, pos: Vec3, color: Color) {
    commands.spawn((
        Text::new(text),
        TextFont { font_size: 26.0, ..default() },
        TextColor(color),
        Transform::from_translation(pos),
        DamageNotif {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            velocity: Vec2::new(0.0, 50.0),
        },
        BattleSprite,
    ));
}

fn spawn_text(commands: &mut Commands, text: &str, pos: Vec3, color: Color) {
    commands.spawn((
        Text::new(text),
        TextFont { font_size: 22.0, ..default() },
        TextColor(color),
        Transform::from_translation(pos),
        DamageNotif {
            timer: Timer::from_seconds(1.3, TimerMode::Once),
            velocity: Vec2::new(0.0, 30.0),
        },
        BattleSprite,
    ));
}

fn spawn_particles(commands: &mut Commands, pos: Vec3, color: Color, count: usize) {
    let mut rng = rand::rng();
    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(60.0..120.0);
        let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            Transform::from_translation(pos),
            Particle {
                timer: Timer::from_seconds(0.6, TimerMode::Once),
                velocity: vel,
            },
            BattleSprite,
        ));
    }
}
