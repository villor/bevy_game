use bevy::prelude::*;
use ldtk_rust::{LayerInstance, TileInstance};
use std::collections::HashMap;
use bevy_rapier2d::prelude::*;

use crate::PHYSICS_SCALE;

const TILE_SCALE: f32 = 2.5;

pub fn load_level(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    project_path: &str,
    level_id: &str
) {
    let project = ldtk_rust::Project::load_project(project_path.to_string());

    // load tilesets
    let mut atlas_handles = HashMap::new();
    for tileset in project.defs.tilesets.iter() {
        let texture_handle = asset_server.load(&tileset.rel_path[..]);
        let texture_atlas_handle = texture_atlases.add(TextureAtlas::from_grid(
            texture_handle,
            Vec2::new(tileset.tile_grid_size as f32, tileset.tile_grid_size as f32),
            (tileset.px_wid / tileset.tile_grid_size) as usize,
            (tileset.px_hei / tileset.tile_grid_size) as usize,
        ));
        atlas_handles.insert(tileset.uid, texture_atlas_handle);
    }
    
    // find level
    let level = project.levels.iter()
        .find(|&level| level.identifier == level_id)
        .unwrap_or_else(|| panic!("level with identifier '{}' does not exist in project", level_id));

    // set bg color
    commands.insert_resource(ClearColor(
        Color::hex(&level.bg_color[1..]).expect("invalid __bg_color hex in json data"),
    ));
    

    for (z_index, layer) in level.layer_instances
        .as_ref()
        .expect("layer instances missing, external levels are not supported")
        .iter()
        .enumerate()
        .rev()
    {
        let tileset_uid = layer.tileset_def_uid.unwrap_or(-1);
        let layer_width = layer.c_wid as f32 * (layer.grid_size as f32 * TILE_SCALE);
        let layer_height = layer.c_hei as f32 * (layer.grid_size as f32 * TILE_SCALE);

        match &layer.layer_instance_type[..] {
            "Tiles" => {
                println!("Spawning Tiles layer: {}", layer.identifier);
                for tile in layer.grid_tiles.iter() {
                    spawn_tile(layer, layer_width, layer_height, tile, z_index as f32, commands, atlas_handles[&tileset_uid].clone());
                }
            },
            "AutoLayer" => {
                println!("Spawning AutoLayer layer: {}", layer.identifier);
                for tile in layer.auto_layer_tiles.iter() {
                    spawn_tile(layer, layer_width, layer_height, tile, z_index as f32, commands, atlas_handles[&tileset_uid].clone());
                }
            },
            "Entities" => {
                println!("Spawning Entities layer: {}", layer.identifier);
                // for entity in layer.entity_instances.iter() {
                //     if entity.identifier == "Player" {
                //         let position = convert_to_world(
                //             layer_width,
                //             layer_height,
                //             layer.grid_size as i32,
                //             TILE_SCALE,
                //             entity.px[0] as i32,
                //             entity.px[1] as i32,
                //             z_index as f32,
                //         );
                //         player::spawn_player(
                //             commands, 
                //             Vec2::new(position.x * PHYSICS_SCALE, position.y * PHYSICS_SCALE)
                //         );
                //     }
                // }
            }
            _ => {
                println!("Skipping layer (not implemented): {}", layer.identifier);
            }
        }
    }
}

fn spawn_tile(
    layer: &LayerInstance,
    layer_width: f32,
    layer_height: f32,
    tile: &TileInstance,
    z_index: f32,
    commands: &mut Commands,
    atlas_handle: Handle<TextureAtlas>,
) {
    let (flip_x, flip_y) = match tile.f {
        1 => (true, false),
        2 => (false, true),
        3 => (true, true),
        _ => (false, false),
    };

    let px_pos = convert_to_world(
        layer_width,
        layer_height,
        layer.grid_size as i32,
        TILE_SCALE,
        tile.px[0] as i32,
        tile.px[1] as i32,
        z_index
    );

    let tile_size = layer.grid_size as f32 * TILE_SCALE;

    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform {
                translation: px_pos,
                ..Default::default()
            },
            sprite: TextureAtlasSprite {
                index: tile.t as usize,
                flip_x,
                flip_y,
                custom_size: Some(Vec2::splat(tile_size)),
                ..Default::default()
            },
            texture_atlas: atlas_handle,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: get_collider_shape(tile.t as i32, tile_size, flip_x, flip_y).into(),
            position: (px_pos / PHYSICS_SCALE).into(),
            flags: ColliderFlags {
                collision_groups: InteractionGroups::new(0b0010, 0b0001),
                ..Default::default()
            }.into(),
            ..Default::default()
        });
}

// LDtk provides pixel locations starting in the top left. For Bevy we need to
// flip the Y axis and offset from the center of the screen.
fn convert_to_world(
    width: f32,
    height: f32,
    grid_size: i32,
    scale: f32,
    x: i32,
    y: i32,
    z: f32,
) -> Vec3 {
    let world_x = (x as f32 * scale) + (grid_size as f32 * scale / 2.) - (width / 2.);
    let world_y = -(y as f32 * scale) - (grid_size as f32 * scale / 2.) + (height / 2.);
    Vec3::new(world_x, world_y, z)
}

enum TileColliderType {
    Square,
    Slope,
    HalfSlope1,
    HalfSlope2,
}

fn get_collider_shape(
    tile_id: i32,
    tile_size: f32,
    flip_x: bool,
    flip_y: bool,
) -> ColliderShape {
    //hardcoded for now
    let (collider_type, tileset_flip_x, tileset_flip_y) = match tile_id {
        // grass
        6 => (TileColliderType::Slope, false, false),
        7 => (TileColliderType::Slope, true, false),
        44 => (TileColliderType::HalfSlope1, false, false),
        45 => (TileColliderType::HalfSlope2, false, false),
        46 => (TileColliderType::HalfSlope2, true, false),
        47 => (TileColliderType::HalfSlope1, true, false),

        // stone
        86 => (TileColliderType::Slope, false, false),
        87 => (TileColliderType::Slope, true, false),
        124 => (TileColliderType::HalfSlope1, false, false),
        125 => (TileColliderType::HalfSlope2, false, false),
        126 => (TileColliderType::HalfSlope2, true, false),
        127 => (TileColliderType::HalfSlope1, true, false),

        _ => (TileColliderType::Square, false, false),
    };

    let physics_size = tile_size / PHYSICS_SCALE;

    let x_flipper = if tileset_flip_x != flip_x { -1.0 } else { 1.0 };
    let y_flipper = if tileset_flip_y != flip_y { -1.0 } else { 1.0 };

    match collider_type {
        TileColliderType::Slope => ColliderShape::convex_hull(&[
            Vec2::new(x_flipper * -physics_size / 2.0, y_flipper * -physics_size / 2.0).into(),
            Vec2::new(x_flipper * physics_size / 2.0, y_flipper * -physics_size / 2.0).into(),
            Vec2::new(x_flipper * -physics_size / 2.0, y_flipper * physics_size / 2.0).into()
        ]).unwrap(),
        TileColliderType::HalfSlope1 => ColliderShape::convex_hull(&[
            Vec2::new(x_flipper * -physics_size / 2.0, y_flipper * -physics_size / 2.0).into(),
            Vec2::new(x_flipper * physics_size / 2.0, y_flipper * -physics_size / 2.0).into(),
            Vec2::new(x_flipper * physics_size / 2.0, 0.0).into(),
            Vec2::new(x_flipper * -physics_size / 2.0, y_flipper * physics_size / 2.0).into()
        ]).unwrap(),
        TileColliderType::HalfSlope2 => ColliderShape::convex_hull(&[
            Vec2::new(x_flipper * -physics_size / 2.0, y_flipper * -physics_size / 2.0).into(),
            Vec2::new(x_flipper * physics_size / 2.0, y_flipper * -physics_size / 2.0).into(),
            Vec2::new(x_flipper * -physics_size / 2.0, 0.0).into()
        ]).unwrap(),
        _ => ColliderShape::cuboid(physics_size / 2.0, physics_size / 2.0),
    }
}
