use bevy::prelude::*;
use crate::components::*;

pub fn update_damage_notifs(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageNotif, &mut Transform, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut notif, mut transform, mut color) in query.iter_mut() {
        notif.timer.tick(time.delta());
        transform.translation.x += notif.velocity.x * time.delta_secs();
        transform.translation.y += notif.velocity.y * time.delta_secs();
        color.0.set_alpha(1.0 - notif.timer.fraction());

        if notif.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut particle, mut transform, mut sprite) in query.iter_mut() {
        particle.timer.tick(time.delta());
        
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();
        
        particle.velocity *= 0.95;
        sprite.color.set_alpha(1.0 - particle.timer.fraction());

        if particle.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_screen_shake(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ScreenShake), With<OverworldCamera>>,
    player_query: Query<&Transform, (With<Player>, Without<OverworldCamera>)>,
    game_state: Res<State<GameState>>,
) {
    let Ok((mut camera_transform, mut shake)) = query.single_mut() else { return };
    shake.trauma = (shake.trauma - time.delta_secs() * 2.5).max(0.0);

    let shake_amount = shake.trauma * shake.trauma;
    let offset_x = (time.elapsed_secs() * 22.0).sin() * shake_amount * 10.0;
    let offset_y = (time.elapsed_secs() * 28.0).cos() * shake_amount * 10.0;

    if *game_state.get() == GameState::Battle {
        camera_transform.translation = Vec3::new(offset_x, BATTLE_ARENA_Y + offset_y, camera_transform.translation.z);
    } else if let Ok(player_transform) = player_query.single() {
        camera_transform.translation.x = offset_x;
        camera_transform.translation.y = player_transform.translation.y + offset_y;
    }
}

pub fn update_battle_ui(
    battle_state: Res<CurrentBattle>,
    player_query: Query<&Player, With<PlayerSprite>>,
    enemy_query: Query<&Enemy>,
    mut text_query: Query<&mut Text, With<HealthText>>,
) {
    let Ok(mut text) = text_query.single_mut() else { return };
    let Ok(player) = player_query.single() else { return };
    let Ok(enemy) = enemy_query.get(battle_state.enemy_entity) else { return };

    **text = format!(
        "♥ Player: {}/{} HP\n◆ Enemy: {}/{} HP",
        player.health.max(0),
        player.max_health,
        enemy.health.max(0),
        enemy.max_health
    );
}

pub fn update_phase_text(
    battle_state: Res<CurrentBattle>,
    mut query: Query<(&mut Text, &mut TextColor), With<PhaseText>>,
) {
    let Ok((mut text, mut color)) = query.single_mut() else { return };

    match battle_state.phase {
        BattlePhase::Intro => {
            **text = "GET READY!".to_string();
            color.0 = Color::srgb(1.0, 1.0, 1.0);
        }
        BattlePhase::PlayerTurn => {
            **text = "YOUR TURN".to_string();
            color.0 = Color::srgb(0.3, 1.0, 0.3);
        }
        BattlePhase::EnemyTelegraph => {
            **text = "INCOMING!".to_string();
            color.0 = Color::srgb(1.0, 0.3, 0.3);
        }
        BattlePhase::BulletHell => {
            **text = "DODGE!".to_string();
            color.0 = Color::srgb(1.0, 0.8, 0.0);
        }
        BattlePhase::Resolution => {
            **text = "".to_string();
        }
    }
}
