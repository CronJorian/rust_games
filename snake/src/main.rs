use bevy::{core::FixedTimestep, prelude::*};
use rand::prelude::random;

const ARENA_HEIGHT: u32 = 10;
const ARENA_WIDTH: u32 = 10;
const BACKGROUND_COLOR: Color = Color::rgb(0.04, 0.04, 0.04);
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
        }
    }
}

#[derive(Component)]
struct Food;

struct GrowthEvent;

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

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SnakeMovement {
    Input,
    Movement,
    Eating,
    Growth,
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Default)]
struct SnakeSegments(Vec<Entity>);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Severus Snek!".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(SnakeSegments::default())
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_event::<GrowthEvent>()
        .add_system(
            snake_movement_input
                .label(SnakeMovement::Input)
                .before(SnakeMovement::Movement),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.15))
                .with_system(snake_movement.label(SnakeMovement::Movement))
                .with_system(
                    snake_eating
                        .label(SnakeMovement::Eating)
                        .after(SnakeMovement::Movement),
                ),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(food_spawner),
        )
        .add_plugins(DefaultPlugins)
        .run();
}

fn food_spawner(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Food)
        .insert(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        })
        .insert(Size::square(0.8));
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

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
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

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_position in head_positions.iter() {
        for (entity, food_position) in food_positions.iter() {
            if food_position == head_position {
                commands.entity(entity).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_position = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Down => {
                head_position.y -= 1;
            }
            Direction::Left => {
                head_position.x -= 1;
            }
            Direction::Right => {
                head_position.x += 1;
            }
            Direction::Up => {
                head_position.y += 1;
            }
        };
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(position, segment)| {
                *positions.get_mut(*segment).unwrap() = *position;
            })
    }
}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let direction: Direction =
            if keyboard_input.any_pressed(vec![KeyCode::Down, KeyCode::S].into_iter()) {
                Direction::Down
            } else if keyboard_input.any_pressed(vec![KeyCode::Left, KeyCode::A].into_iter()) {
                Direction::Left
            } else if keyboard_input.any_pressed(vec![KeyCode::Right, KeyCode::D].into_iter()) {
                Direction::Right
            } else if keyboard_input.any_pressed(vec![KeyCode::Up, KeyCode::W].into_iter()) {
                Direction::Up
            } else {
                head.direction
            };
        if direction != head.direction.opposite() {
            head.direction = direction;
        }
    }
}

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    segments.0 = vec![
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(SnakeHead {
                direction: Direction::Up,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
        spawn_snake_segment(commands, Position { x: 3, y: 2 }),
    ];
}

fn spawn_snake_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}
