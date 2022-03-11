use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};
use bevy_rapier2d::prelude::*;
use bevy_prototype_debug_lines::*;

mod math;
mod level;
mod player;
mod platformer;

pub const PHYSICS_SCALE: f32 = 40.0; // 1m = 40px

fn main() {
    App::new()
        //.insert_resource(ClearColor(Color::rgb(0.086, 0.3, 0.67)))
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .insert_resource(RapierConfiguration {
            scale: PHYSICS_SCALE,
            physics_pipeline_active: false,
            ..Default::default()
        })
        .register_inspectable::<platformer::PlatformerController>()
        .register_inspectable::<platformer::PlatformerRaycaster>()
        .register_inspectable::<platformer::PlatformerCollisionInfo>()
        .add_startup_system(setup)
        .add_system(player::get_keyboard_input)
        .add_system(platformer::platformer_controller_update.label("platformer_pre_update"))
        .add_system(platformer::update_raycaster.label("platformer_pre_update"))
        .add_system(platformer::platformer_check_collisions.label("platformer_collisions").after("platformer_pre_update"))
        .add_system(platformer::platformer_move.label("platformer_move").after("platformer_collisions"))
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    level::load_level(&mut commands, &asset_server, &mut texture_atlases, "assets/map.ldtk", "Level_0");
    player::spawn_player(&mut commands, Vec2::splat(0.0));
}

