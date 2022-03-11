use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::PHYSICS_SCALE;
use crate::platformer::*;

pub fn spawn_player(commands: &mut Commands, position: Vec2) {  
    const PLAYER_HEIGHT: f32 = 1.5;
    const PLAYER_WIDTH: f32 = 1.0;
    commands
        .spawn_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0).into(),
            position: position.into(),
            flags: ColliderFlags {
                collision_groups: InteractionGroups::new(0b0000, 0b0000),
                ..Default::default()
            }.into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        //.insert(ColliderDebugRender::with_id(1))
        .insert_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(PLAYER_WIDTH * PHYSICS_SCALE, PLAYER_HEIGHT * PHYSICS_SCALE)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PlatformerRaycaster::default())
        .insert(PlatformerMoveDelta::default())
        .insert(PlatformerController::default())
        .insert(PlatformerCollisionInfo::default())
        .insert(PlatformerInput::default());
}

pub fn get_keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_inputs: Query<&mut PlatformerInput>,
) {
    for mut player_input in player_inputs.iter_mut() {
        player_input.x_movement = 0.0;
        if keyboard_input.pressed(KeyCode::A) {
            player_input.x_movement -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            player_input.x_movement += 1.0;
        }

        player_input.jumping = keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::W);
    }
}
