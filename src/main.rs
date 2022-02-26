use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.086, 0.3, 0.67)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(player_movement)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(mut commands: Commands) {
    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // player
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -215.0, 0.0),
                scale: Vec3::new(30.0, 40.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player);
    
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -315.0, 0.0),
                scale: Vec3::new(2500.0, 40.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.1, 0.1, 0.1),
                ..Default::default()
            },
            ..Default::default()
        });
}

#[derive(Component)]
struct Player {
    max_speed: f32,
    acceleration: f32,
    deceleration: f32,
    horizontal_speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            max_speed: 8.0,
            acceleration: 0.8,
            deceleration: 0.4,
            horizontal_speed: 0.0
        }
    }
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let (_, mut transform) = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        transform.translation.x -= 200.0 * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::Right) {
        transform.translation.x += 200.0 * time.delta_seconds();
    }

    transform.translation.x 
}
