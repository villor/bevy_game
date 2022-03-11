use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;
use bevy_prototype_debug_lines::*;

use crate::PHYSICS_SCALE;
use crate::math;

#[derive(Component, Default)]
pub struct PlatformerInput {
    pub x_movement: f32,
    pub jumping: bool,
}

#[derive(Default, Inspectable)]
pub struct RaycastOrigins {
    top_left: Vec2,
    top_right: Vec2,
    bottom_left: Vec2,
    bottom_right: Vec2,
}

#[derive(Component, Inspectable)]
pub struct PlatformerRaycaster {
    margin: f32,
    horizontal_ray_count: u32,
    vertical_ray_count: u32,
    horizontal_ray_spacing: f32,
    vertical_ray_spacing: f32,
    origins: RaycastOrigins,
}

impl Default for PlatformerRaycaster {
    fn default() -> Self {
        Self {
            margin: 0.015,
            horizontal_ray_count: 4,
            vertical_ray_count: 4,
            horizontal_ray_spacing: 0.0,
            vertical_ray_spacing: 0.0,
            origins: Default::default(),
        }
    }
}

pub fn update_raycaster(
    mut query: Query<(&mut PlatformerRaycaster, &ColliderPositionComponent, &ColliderShapeComponent)>,
) {
    for (mut raycaster, position, shape) in query.iter_mut() {
        // Clamp ray count
        raycaster.horizontal_ray_count = raycaster.horizontal_ray_count.clamp(2, u32::MAX);
        raycaster.vertical_ray_count = raycaster.vertical_ray_count.clamp(2, u32::MAX);

        // Get AABB for the collider and make it smaller by margin
        let  bounds = shape.compute_aabb(position);
        let maxs: Vec2 = Vec2::from(bounds.maxs) - Vec2::splat(raycaster.margin);
        let mins: Vec2 = Vec2::from(bounds.mins) + Vec2::splat(raycaster.margin);

        // Calculate ray spacing
        raycaster.horizontal_ray_spacing = (maxs.y - mins.y) / (raycaster.horizontal_ray_count - 1) as f32;
        raycaster.vertical_ray_spacing = (maxs.x - mins.x) / (raycaster.vertical_ray_count - 1) as f32;

        // Update origins
        raycaster.origins.bottom_left = Vec2::new(mins.x, mins.y);
        raycaster.origins.bottom_right = Vec2::new(maxs.x, mins.y);
        raycaster.origins.top_left = Vec2::new(mins.x, maxs.y);
        raycaster.origins.top_right = Vec2::new(maxs.x, maxs.y);
    }
}

#[derive(Component, Inspectable)]
pub struct PlatformerController {
    max_speed: f32,
    acceleration: f32,
    deceleration: f32,
    gravity: f32,
    velocity: Vec2,
}

impl Default for PlatformerController {
    fn default() -> Self {
        Self {
            max_speed: 5.0,
            acceleration: 0.1,
            deceleration: 0.1,
            gravity: 15.0,
            velocity: Vec2::ZERO,
        }
    }
}

#[derive(Component, Default)]
pub struct PlatformerMoveDelta(Vec2);


pub fn platformer_controller_update(
    time: Res<Time>,
    mut query: Query<(&mut PlatformerController, &PlatformerInput, &mut PlatformerMoveDelta, &PlatformerCollisionInfo)>,
) {
    for (mut controller, input, mut delta, collision_info) in query.iter_mut() {
        if collision_info.above || collision_info.below {
            controller.velocity.y = 0.0;
        }

        if input.jumping && collision_info.below {
            controller.velocity.y = 8.0;
        }

        // Apply gravity
        controller.velocity.y -= controller.gravity * time.delta_seconds();

        // Horizontal movement
        if input.x_movement != 0.0 {
            // speed up
            controller.velocity.x += input.x_movement * (controller.max_speed / controller.acceleration) * time.delta_seconds();
            controller.velocity.x = controller.velocity.x.clamp(-controller.max_speed, controller.max_speed);
        } else {
            // slow down
            controller.velocity.x = math::move_towards(controller.velocity.x, 0.0, (controller.max_speed / controller.deceleration) * time.delta_seconds());
        }

        delta.0 = controller.velocity * time.delta_seconds();
    }
}

#[derive(Component, Default, Inspectable)]
pub struct PlatformerCollisionInfo {
    above: bool,
    below: bool,
    left: bool,
    right: bool,
}

impl PlatformerCollisionInfo {
    fn reset(&mut self) {
        self.above = false;
        self.below = false;
        self.left = false;
        self.right = false;
    }
}

pub fn platformer_check_collisions(
    //time: Res<Time>,
    query_pipeline: Res<QueryPipeline>,
    collider_query: QueryPipelineColliderComponentsQuery,
    mut query: Query<(&mut PlatformerMoveDelta, &mut PlatformerCollisionInfo, &PlatformerRaycaster)>,
    mut debug_lines: ResMut<DebugLines>,
) {
    let collider_set = QueryPipelineColliderComponentsSet(&collider_query);

    for (mut delta, mut collision_info, raycaster) in query.iter_mut() {
        collision_info.reset();
        
        // Horizontal collisions
        if delta.0.x != 0.0
        {
            let direction_x = delta.0.x.signum();
            let mut ray_length = delta.0.x.abs() + raycaster.margin;
            let first_origin = if direction_x == -1.0 { raycaster.origins.bottom_left } else { raycaster.origins.bottom_right };

            for i in 0..raycaster.vertical_ray_count {
                let ray_origin = first_origin + Vec2::new(0.0, 1.0) * (raycaster.horizontal_ray_spacing * i as f32);
                let ray = Ray::new(ray_origin.into(), (Vec2::new(1.0, 0.0) * direction_x).into());
                let hit = query_pipeline.cast_ray(
                    &collider_set,
                    &ray,
                    ray_length,
                    true,
                    InteractionGroups::new(0b0001, 0b0010),
                    None
                );

                if let Some((_, toi)) = hit {
                    delta.0.x = (toi - raycaster.margin) * direction_x;
                    ray_length = toi;

                    collision_info.left = direction_x == -1.0;
                    collision_info.right = direction_x == 1.0;
                }

                debug_lines.line((ray_origin * PHYSICS_SCALE, 5.0).into(), ((ray_origin + Vec2::new(ray_length * direction_x, 0.0)) * PHYSICS_SCALE, 5.0).into(), 0.0);
            }
        }

        // Vertical collisions
        if delta.0.y != 0.0
        {
            let direction_y = delta.0.y.signum();
            let mut ray_length = delta.0.y.abs() + raycaster.margin;
            let first_origin = if direction_y == -1.0 { raycaster.origins.bottom_left } else { raycaster.origins.top_left };

            for i in 0..raycaster.vertical_ray_count {
                let ray_origin = first_origin + Vec2::new(1.0, 0.0) * (raycaster.vertical_ray_spacing * i as f32 + delta.0.x);
                let ray = Ray::new(ray_origin.into(), (Vec2::new(0.0, 1.0) * direction_y).into());
                let hit = query_pipeline.cast_ray(
                    &collider_set,
                    &ray,
                    ray_length,
                    true,
                    InteractionGroups::new(0b0001, 0b0010),
                    None
                );

                if let Some((_, toi)) = hit {
                    delta.0.y = (toi - raycaster.margin) * direction_y;
                    ray_length = toi;

                    collision_info.below = direction_y == -1.0;
                    collision_info.above = direction_y == 1.0;
                }

                debug_lines.line((ray_origin * PHYSICS_SCALE, 0.0).into(), ((ray_origin + Vec2::new(0.0, ray_length * direction_y)) * PHYSICS_SCALE, 0.0).into(), 0.0);
            }
        }
    }
}

pub fn platformer_move(
    mut query: Query<(&PlatformerMoveDelta, &mut ColliderPositionComponent)>,
) {
    for (delta, mut position) in query.iter_mut() {
        position.translation = Vec2::new(position.translation.x + delta.0.x, position.translation.y + delta.0.y).into();
    }
}
