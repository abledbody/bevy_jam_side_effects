use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::actor::enemy::EnemyTemplate;
use crate::game::actor::player::PlayerTemplate;
use crate::game::actor::player::Playthrough;
use crate::game::actor::ActorAssets;
use crate::game::level::exit::ExitTemplate;
use crate::game::level::gate::GateAssets;
use crate::game::level::gate::GateTemplate;
use crate::game::level::plate::PlateAssets;
use crate::game::level::plate::PlateTemplate;
use crate::game::level::victory::VictorySquareTemplate;
use crate::game::level::wall::WallTemplate;
use crate::util::vfx::VfxAssets;
use crate::util::DespawnSet;

mod exit;
mod gate;
pub mod plate;
pub mod victory;
mod wall;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            level_background: LevelBackground::Nonexistent,
            ..default()
        })
        .init_resource::<LevelSelection>()
        .add_plugins(LdtkPlugin);

        app.register_type::<LevelAssets>()
            .init_collection::<LevelAssets>();

        app.add_systems(PreUpdate, populate_level);

        app.add_plugins((
            exit::ExitPlugin,
            gate::GatePlugin,
            plate::PlatePlugin,
            victory::VictoryPlugin,
            wall::WallPlugin,
        ));
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[asset(path = "level/main.ldtk")]
    main: Handle<LdtkProject>,
}

pub struct LevelTemplate;

impl LevelTemplate {
    pub fn spawn(self, commands: &mut Commands, level_assets: &LevelAssets) -> Entity {
        commands
            .spawn((
                Name::new("Level"),
                LdtkWorldBundle {
                    ldtk_handle: level_assets.main.clone(),
                    ..default()
                },
            ))
            .id()
    }
}

pub fn populate_level(
    mut commands: Commands,
    mut despawn: ResMut<DespawnSet>,
    actor_assets: Res<ActorAssets>,
    gate_assets: Res<GateAssets>,
    plate_assets: Res<PlateAssets>,
    vfx_assets: Res<VfxAssets>,
    entity_query: Query<(Entity, &Parent, &Transform, &EntityInstance), Added<EntityInstance>>,
    tile_query: Query<(&Parent, &Transform, &TileEnumTags), Added<TileEnumTags>>,
    playthrough: Res<Playthrough>,
) {
    let mut gate_map = HashMap::new();

    for (_, parent, &transform, instance) in &entity_query {
        let entity = match instance.identifier.as_str() {
            "player" => PlayerTemplate {
                transform,
                current_health: playthrough.health.unwrap_or(200.0),
                texture: if playthrough.defected {
                    actor_assets.gnoll_blue.clone()
                } else {
                    actor_assets.gnoll_red.clone()
                },
                ..default()
            }
            .spawn(&mut commands, &actor_assets, &vfx_assets),
            "enemy" => EnemyTemplate {
                transform,
                ..default()
            }
            .with_random_name()
            .spawn(&mut commands, &actor_assets, &vfx_assets),
            "corpse" => EnemyTemplate {
                transform,
                ..default()
            }
            .with_random_name()
            .dead()
            .spawn(&mut commands, &actor_assets, &vfx_assets),
            "open_gate" => {
                let gate = GateTemplate {
                    transform,
                    open: true,
                }
                .spawn(&mut commands, &gate_assets);

                gate_map.insert(&instance.iid, gate);
                gate
            },
            "closed_gate" => {
                let gate = GateTemplate {
                    transform,
                    open: false,
                }
                .spawn(&mut commands, &gate_assets);

                gate_map.insert(&instance.iid, gate);
                gate
            },
            "exit" => ExitTemplate { transform }.spawn(&mut commands),
            "victory" => VictorySquareTemplate { transform }.spawn(&mut commands),
            _ => continue,
        };
        commands.entity(parent.get()).add_child(entity);
    }

    // Spawn plates last so they can link to gates by Entity
    for (entity, parent, &transform, instance) in &entity_query {
        // Despawn marker entity
        despawn.recursive(entity);

        let entity = match instance.identifier.as_str() {
            "plate" => {
                let mut gates = vec![];
                for field in &instance.field_instances {
                    if field.identifier != "gates" {
                        continue;
                    }
                    let FieldValue::EntityRefs(entity_refs) = &field.value else {
                        continue;
                    };

                    gates.extend(
                        entity_refs
                            .iter()
                            .filter_map(|x| x.as_ref())
                            .filter_map(|entity_ref| gate_map.get(&entity_ref.entity_iid)),
                    );
                    break;
                }

                PlateTemplate { transform, gates }.spawn(&mut commands, &plate_assets)
            },
            _ => continue,
        };
        commands.entity(parent.get()).add_child(entity);
    }

    for (parent, &transform, tile) in &tile_query {
        if !tile.tags.iter().any(|s| s.as_str() == "wall") {
            continue;
        }

        let entity = WallTemplate { transform }.spawn(&mut commands);
        commands.entity(parent.get()).add_child(entity);
    }
}
