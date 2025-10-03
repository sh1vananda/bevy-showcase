use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Overworld,
    Battle,
    GameOver,
    Victory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattlePhase {
    Intro,
    PlayerTurn,
    EnemyTelegraph,
    BulletHell,
    Resolution,
}

pub const PLAYER_SPEED: f32 = 180.0;
pub const PLAYER_MAX_HEALTH: i32 = 30;
pub const ROOM_HEIGHT: f32 = 140.0;
pub const TOTAL_ROOMS: usize = 6;
pub const BATTLE_ARENA_Y: f32 = 250.0;
pub const ARENA_WIDTH: f32 = 350.0;
pub const ARENA_HEIGHT: f32 = 280.0;

#[derive(Component)]
pub struct Player {
    pub health: i32,
    pub max_health: i32,
}

#[derive(Component)]
pub struct Enemy {
    pub health: i32,
    pub max_health: i32,
    pub room_index: usize,
    pub attack_pattern: usize,
}

#[derive(Component)]
pub struct Room {
    pub index: usize,
    pub cleared: bool,
}

#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
    pub damage: i32,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct BattleSprite;

#[derive(Component)]
pub struct PlayerSprite;

#[derive(Component)]
pub struct EnemySprite;

#[derive(Component)]
pub struct DamageNotif {
    pub timer: Timer,
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct ScreenShake {
    pub trauma: f32,
}

#[derive(Component)]
pub struct AttackIndicator {
    pub speed: f32,
    pub direction: f32,
}

#[derive(Component)]
pub struct Telegraph {
    pub timer: Timer,
}

#[derive(Component)]
pub struct ExitDoor;

#[derive(Component)]
pub struct BattleUI;

#[derive(Component)]
pub struct PhaseText;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct ControlsText;

#[derive(Component)]
pub struct OverworldInstructions;

#[derive(Component)]
pub struct RoomCounter;

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub struct OverworldCamera;

#[derive(Component)]
pub struct Particle {
    pub timer: Timer,
    pub velocity: Vec2,
}

#[derive(Resource)]
pub struct CurrentBattle {
    pub enemy_entity: Entity,
    pub phase: BattlePhase,
    pub phase_timer: Timer,
    pub player_defended: bool,
    pub combo_count: usize,
}

#[derive(Resource)]
pub struct GameProgress {
    pub current_room: usize,
    pub rooms_cleared: usize,
    pub total_rooms: usize,
}

#[derive(Resource)]
pub struct BulletSpawner {
    pub timer: Timer,
}