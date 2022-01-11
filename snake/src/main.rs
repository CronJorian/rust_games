use bevy::prelude::*;

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const ARENA_HEIGHT: u32 = 10;
const ARENA_WIDTH: u32 = 10;

#[derive(Component)]
struct SnakeHead;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

fn main() {
    App::new()
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_system(snake_movement)
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_snake(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_HEAD_COLOR,
                ..Default::default()
            },
            transform: Transform {
                scale: Vec3::new(10.0, 10.0, 10.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SnakeHead)
        .insert(Position { x: 3, y: 3 })
        .insert(Size::square(0.8));
}

fn snake_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut head_positions: Query<&mut Transform, With<SnakeHead>>,
) {
    for mut transform in head_positions.iter_mut() {
        if keyboard_input.any_pressed(vec![KeyCode::Left, KeyCode::A].into_iter()) {
            transform.translation.x -= 2.;
        }
        if keyboard_input.any_pressed(vec![KeyCode::Right, KeyCode::D].into_iter()) {
            transform.translation.x += 2.;
        }
        if keyboard_input.any_pressed(vec![KeyCode::Down, KeyCode::S].into_iter()) {
            transform.translation.y -= 2.;
        }
        if keyboard_input.any_pressed(vec![KeyCode::Up, KeyCode::W].into_iter()) {
            transform.translation.y += 2.;
        }
    }
}

fn size_scaling(windows: Res<Windows>, mut query: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap(); // TODO: Remove unwrap and use matching pattern for Some/None
    for (sprite_size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.,
        )
    }
}

fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let window = windows.get_primary().unwrap(); // TODO: Remove unwrap and use matching pattern for Some/None
    for (position, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(position.x as f32, window.width(), ARENA_WIDTH as f32),
            convert(position.y as f32, window.height(), ARENA_HEIGHT as f32),
            0.,
        )
    }
}
