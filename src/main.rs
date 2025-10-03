use bevy::prelude::*;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, setup_tilemap))
        .add_systems(Update, (player_movement, camera_follow))
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.transform.scale = Vec3::new(2.0, 2.0, 1.0);
    commands.spawn(camera);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        },
        Player,
    ));

    // Signpost
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.5, 0.25, 0.0),
            custom_size: Some(Vec2::new(16.0, 16.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(48.0, 0.0, 0.0)),
        ..default()
    });

    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "Hello, Bevy!",
            TextStyle {
                font_size: 10.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        transform: Transform::from_translation(Vec3::new(48.0, 24.0, 1.0)),
        ..default()
    });
}

fn setup_tilemap(mut commands: Commands) {
    let tile_size = 16.0;
    let map_size = 20;

    for x in -map_size..=map_size {
        for y in -map_size..=map_size {
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.5, 0.5),
                    custom_size: Some(Vec2::new(tile_size, tile_size)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * tile_size,
                    y as f32 * tile_size,
                    0.0,
                )),
                ..default()
            });
        }
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut transform = query.single_mut();
    let speed = 100.0;

    if keyboard_input.pressed(KeyCode::KeyA) {
        transform.translation.x -= speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        transform.translation.x += speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        transform.translation.y += speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        transform.translation.y -= speed * time.delta_seconds();
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}
