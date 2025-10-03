use bevy::prelude::*;
use crate::components::*;

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = query.single_mut() else { return };

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
        transform.translation.x += direction.x * PLAYER_SPEED * time.delta_secs();
        transform.translation.y += direction.y * PLAYER_SPEED * time.delta_secs();
    }

    transform.translation.x = transform.translation.x.clamp(-110.0, 110.0);
}

pub fn check_room_transition(
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Enemy), Without<Player>>,
    rooms_query: Query<&Room>,
    mut battle_state: ResMut<CurrentBattle>,
    mut game_state: ResMut<NextState<GameState>>,
    mut battle_ui: Query<&mut Visibility, With<BattleUI>>,
    mut game_progress: ResMut<GameProgress>,
) {
    let Ok(player_transform) = player_query.single() else { return };

    for (enemy_entity, enemy_transform, enemy) in enemy_query.iter() {
        let distance = player_transform.translation.distance(enemy_transform.translation);
        let room_cleared = rooms_query.iter().any(|room| room.index == enemy.room_index && room.cleared);

        if distance < 40.0 && enemy.health > 0 && !room_cleared {
            battle_state.enemy_entity = enemy_entity;
            battle_state.phase = BattlePhase::Intro;
            battle_state.phase_timer = Timer::from_seconds(0.8, TimerMode::Once);
            battle_state.player_defended = false;
            battle_state.combo_count = 0;
            game_progress.current_room = enemy.room_index;

            for mut visibility in battle_ui.iter_mut() {
                *visibility = Visibility::Visible;
            }

            game_state.set(GameState::Battle);
            break;
        }
    }
}

pub fn check_exit_door(
    player_query: Query<&Transform, With<Player>>,
    exit_query: Query<&Transform, (With<ExitDoor>, Without<Player>)>,
    game_progress: Res<GameProgress>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(exit_transform) = exit_query.single() else { return };
    let distance = player_transform.translation.distance(exit_transform.translation);

    if distance < 45.0 && game_progress.rooms_cleared >= game_progress.total_rooms {
        game_state.set(GameState::Victory);
    }
}

pub fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<OverworldCamera>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return };

    // Smooth lerp camera
    let target_y = player_transform.translation.y;
    camera_transform.translation.y += (target_y - camera_transform.translation.y) * 0.1;
    camera_transform.translation.x = 0.0;
}
