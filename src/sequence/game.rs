use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::LevelSelection;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::prelude::*;

use crate::common::camera::CameraRoot;
use crate::common::UpdateSet;
use crate::game::actor::enemy::AlertEvent;
use crate::game::actor::player::Playthrough;
use crate::game::alarm::Alarm;
use crate::game::alarm::AlarmAssets;
use crate::game::alarm::AlarmMeterTemplate;
use crate::game::combat::DeathEvent;
use crate::game::combat::HitEvent;
use crate::game::cutscene::CutsceneAssets;
use crate::game::cutscene::CutsceneTemplate;
use crate::game::level::victory::Victory;
use crate::game::level::LevelAssets;
use crate::game::level::LevelTemplate;
use crate::sequence::SequenceState;
use crate::sequence::SequenceState::*;
use crate::util::ui::UiRoot;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameAssets>()
            .init_collection::<GameAssets>();

        app.register_type::<GameRoot>().init_resource::<GameRoot>();

        app.add_systems(OnEnter(Game), enter_game)
            .add_systems(OnExit(Game), exit_game)
            .add_systems(OnEnter(RestartGame), |mut state: ResMut<NextState<_>>| {
                state.set(Game);
            });

        app.init_resource::<ActionState<GameAction>>()
            .insert_resource(
                InputMap::default()
                    .insert(GameAction::Restart, KeyCode::KeyR)
                    .insert(GameAction::Confirm, KeyCode::Space)
                    .insert(GameAction::Confirm, KeyCode::Enter)
                    .insert(GameAction::Confirm, MouseButton::Left)
                    .build(),
            )
            .add_plugins(InputManagerPlugin::<GameAction>::default())
            .add_systems(
                Update,
                restart
                    .in_set(UpdateSet::HandleActions)
                    .run_if(in_state(Game).and_then(action_just_pressed(GameAction::Restart))),
            );
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
struct GameAssets {
    #[asset(path = "sound/sfx/pop_1.wav")]
    sfx_restart: Handle<AudioSource>,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GameRoot {
    pub game: Entity,
}

impl FromWorld for GameRoot {
    fn from_world(world: &mut World) -> Self {
        let game = world
            .spawn((Name::new("Game"), SpatialBundle::default()))
            .id();

        Self { game }
    }
}

fn enter_game(
    mut commands: Commands,
    alarm_assets: Res<AlarmAssets>,
    cutscene_assets: Res<CutsceneAssets>,
    level_assets: Res<LevelAssets>,
    game_root: Res<GameRoot>,
    ui_root: Res<UiRoot>,
    mut seen_cutscene: Local<bool>,
) {
    // Spawn level
    let level = LevelTemplate.spawn(&mut commands, &level_assets);
    commands.entity(level).set_parent(game_root.game);

    // Spawn HUD
    let alarm_meter = AlarmMeterTemplate.spawn(&mut commands, &alarm_assets);
    commands.entity(alarm_meter).set_parent(ui_root.body);

    // Spawn cutscene only on the first run
    if !*seen_cutscene {
        *seen_cutscene = true;

        let cutscene = CutsceneTemplate.spawn(&mut commands, &cutscene_assets);
        commands.entity(cutscene).set_parent(ui_root.body);
    }
}

fn exit_game(
    mut commands: Commands,
    game_root: Res<GameRoot>,
    ui_root: Res<UiRoot>,
    camera_root: Res<CameraRoot>,
    mut collision_events: ResMut<Events<CollisionEvent>>,
    mut hit_events: ResMut<Events<HitEvent>>,
    mut death_events: ResMut<Events<DeathEvent>>,
    mut detect_events: ResMut<Events<AlertEvent>>,
    mut level_selection: ResMut<LevelSelection>,
    mut playthrough: ResMut<Playthrough>,
    mut victory: ResMut<Victory>,
    mut alarm: ResMut<Alarm>,
    mut camera_query: Query<&mut Transform>,
) {
    // Reset resources
    *level_selection = default();
    *playthrough = default();
    *victory = default();
    *alarm = default();

    // Clear events
    collision_events.clear();
    hit_events.clear();
    death_events.clear();
    detect_events.clear();

    // Despawn entities
    commands.entity(ui_root.body).despawn_descendants();
    commands.entity(game_root.game).despawn_descendants();

    // Reset camera
    if let Ok(mut transform) = camera_query.get_mut(camera_root.primary) {
        transform.translation = Vec2::ZERO.extend(transform.translation.z);
    };
}

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq)]
pub enum GameAction {
    Restart,
    // TODO: This should be CutsceneAction::Advance, and a component not a resource
    Confirm,
}

fn restart(
    mut state: ResMut<NextState<SequenceState>>,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
) {
    state.set(RestartGame);

    // Play restart sound
    audio.play(game_assets.sfx_restart.clone());
}
