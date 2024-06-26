pub mod ball;
mod brick;
mod collectable;
pub mod collider;
pub mod events;
mod paddle;
mod pause_view;
mod preparation_view;
pub mod resources;
mod score_view;
mod shared;
mod spark;
mod summary_view;

use crate::AppState;
use bevy::prelude::*;

use crate::game::ball::{keep_ball_synced_with_settings, keep_destroying_balls};
use crate::game::brick::{keep_brick_synced_with_settings, keep_spawning_bricks};
use crate::game::collectable::{despawn_collectables, keep_spawning_collectables};
use crate::game::events::{
    BrickDestroyed, LastBallDestroyed, MenuRequested, RestartRequested, TogglePauseRequested,
};
use crate::game::pause_view::{check_pause_interactions, despawn_pause_view, spawn_pause_view};
use crate::game::preparation_view::{despawn_preparation_view, spawn_preparation_view};
use crate::game::resources::{
    BallSize, BallSpeed, BrickGhost, BrickRowSpawnCooldown, PaddleSize, PaddleSpeed, Score,
};
use crate::game::score_view::{despawn_score_view, spawn_score_view, update_score_view};
use crate::game::shared::{collect_collectables, keep_ball_at_paddle_center};
use crate::game::spark::{keep_despawning_sparks, move_sparks};
use crate::game::summary_view::{
    check_summary_interactions, despawn_summary_view, spawn_summary_view,
};
use ball::{despawn_balls, move_balls, spawn_first_ball};
use brick::{despawn_bricks, destroy_bricks_on_hit, spawn_bricks};
use paddle::{despawn_paddles, keep_paddle_synced_with_settings, move_paddle, spawn_paddle};

pub struct GamePlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum InGameState {
    #[default]
    None,
    Preparation,
    Play,
    Pause,
    Summary,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InGameState>()
            .init_resource::<Score>()
            .init_resource::<BrickRowSpawnCooldown>()
            .init_resource::<BallSize>()
            .init_resource::<BallSpeed>()
            .init_resource::<BrickGhost>()
            .init_resource::<PaddleSize>()
            .init_resource::<PaddleSpeed>()
            .add_event::<BrickDestroyed>()
            .add_event::<LastBallDestroyed>()
            .add_event::<RestartRequested>()
            .add_event::<MenuRequested>()
            .add_event::<TogglePauseRequested>()
            .add_systems(OnEnter(AppState::RestartInGame), continue_restart_game)
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    spawn_score_view,
                    spawn_first_ball,
                    spawn_paddle,
                    spawn_bricks,
                    start_up,
                ),
            )
            .add_systems(
                OnExit(AppState::InGame),
                (
                    despawn_score_view,
                    despawn_balls,
                    despawn_paddles,
                    despawn_bricks,
                    despawn_collectables,
                    clean_up,
                ),
            )
            .add_systems(OnEnter(InGameState::Preparation), spawn_preparation_view)
            .add_systems(OnExit(InGameState::Preparation), despawn_preparation_view)
            .add_systems(OnEnter(InGameState::Pause), spawn_pause_view)
            .add_systems(OnExit(InGameState::Pause), despawn_pause_view)
            .add_systems(OnEnter(InGameState::Summary), spawn_summary_view)
            .add_systems(OnExit(InGameState::Summary), despawn_summary_view)
            .add_systems(
                Update,
                (
                    (
                        (move_paddle, keep_ball_at_paddle_center).chain(),
                        check_preparation_end_condition,
                    )
                        .run_if(in_state(InGameState::Preparation)),
                    (
                        update_score_view,
                        move_paddle,
                        move_balls,
                        destroy_bricks_on_hit,
                        test_settings,
                        keep_ball_synced_with_settings,
                        keep_destroying_balls,
                        keep_paddle_synced_with_settings,
                        keep_brick_synced_with_settings,
                        keep_spawning_bricks,
                        keep_spawning_collectables,
                        move_sparks,
                        keep_despawning_sparks,
                        collect_collectables,
                    )
                        .run_if(in_state(InGameState::Play)),
                    (check_pause_interactions,).run_if(in_state(InGameState::Pause)),
                    (check_summary_interactions,).run_if(in_state(InGameState::Summary)),
                    (
                        check_menu_condition,
                        check_restart_condition,
                        check_toggle_pause_condition,
                        check_summary_condition,
                    ),
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn start_up(mut next_state: ResMut<NextState<InGameState>>) {
    next_state.set(InGameState::Preparation);
}

fn clean_up(mut commands: Commands, mut next_state: ResMut<NextState<InGameState>>) {
    next_state.set(InGameState::None);
    commands.insert_resource(Score::default());
    commands.insert_resource(BrickRowSpawnCooldown::default());
    commands.insert_resource(BallSize::default());
    commands.insert_resource(BallSpeed::default());
    commands.insert_resource(BrickGhost::default());
    commands.insert_resource(PaddleSize::default());
    commands.insert_resource(PaddleSpeed::default());
}

fn check_preparation_end_condition(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    if let Some(key) = keyboard_input.get_just_pressed().next() {
        if *key != KeyCode::ArrowLeft && *key != KeyCode::ArrowRight {
            next_state.set(InGameState::Play);
        }
    } else if mouse_input.get_just_pressed().next() != None {
        next_state.set(InGameState::Play);
    }
}

fn check_menu_condition(
    mut menu_requested_events: EventReader<MenuRequested>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if menu_requested_events.is_empty() {
        return;
    }

    menu_requested_events.clear();
    next_state.set(AppState::Menu);
}

fn check_restart_condition(
    mut restart_requested_events: EventReader<RestartRequested>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if restart_requested_events.is_empty() {
        return;
    }

    restart_requested_events.clear();
    next_state.set(AppState::RestartInGame);
}

fn continue_restart_game(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InGame);
}

fn check_summary_condition(
    mut last_ball_destroyed_events: EventReader<LastBallDestroyed>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    if last_ball_destroyed_events.is_empty() {
        return;
    }

    last_ball_destroyed_events.clear();
    next_state.set(InGameState::Summary);
}

fn check_toggle_pause_condition(
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<InGameState>>,
    mut next_state: ResMut<NextState<InGameState>>,
    mut toggle_pause_requested_events: EventReader<TogglePauseRequested>,
) {
    let mut toggle = false;

    if !toggle_pause_requested_events.is_empty() {
        toggle = true;
        toggle_pause_requested_events.clear();
    }
    if input.just_pressed(KeyCode::Escape) {
        toggle = true;
    }

    if !toggle {
        return;
    }

    if *current_state.get() == InGameState::Play {
        next_state.set(InGameState::Pause);
    } else {
        next_state.set(InGameState::Play);
    }
}

pub fn test_settings(
    input: Res<ButtonInput<KeyCode>>,
    mut ball_size: ResMut<BallSize>,
    mut ball_speed: ResMut<BallSpeed>,
    mut brick_ghost: ResMut<BrickGhost>,
    mut paddle_size: ResMut<PaddleSize>,
    mut paddle_speed: ResMut<PaddleSpeed>,
) {
    let value = if input.just_pressed(KeyCode::KeyQ) {
        -1
    } else if input.just_pressed(KeyCode::KeyE) {
        1
    } else {
        0
    };

    if value == 0 {
        return;
    }

    if input.pressed(KeyCode::Digit1) {
        paddle_size.change_points(value);
    }
    if input.pressed(KeyCode::Digit2) {
        paddle_speed.change_points(value);
    }
    if input.pressed(KeyCode::Digit3) {
        ball_size.change_points(value);
    }
    if input.pressed(KeyCode::Digit4) {
        ball_speed.change_points(value);
    }
    if input.pressed(KeyCode::Digit5) {
        brick_ghost.set_enabled(value > 0);
    }
}
