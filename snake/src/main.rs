use bevy::{core::FixedTimestep, prelude::*};
use rand::random;
use std::process;

const ARENA_HEIGHT: u32 = 10;
const ARENA_WIDTH: u32 = 10;
const BACKGROUND_COLOR: Color = Color::rgb(0.04, 0.04, 0.04);
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    None,
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
        }
    }
}

#[derive(Component)]
struct Food;

struct GameOverEvent;

struct GrowthEvent;

#[derive(Default)]
struct LastTailPosition(Option<Position>);

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
        .insert_resource(LastTailPosition::default())
        .add_startup_system(setup_camera)
        .add_startup_system(snake_spawner)
        .add_event::<GameOverEvent>()
        .add_event::<GrowthEvent>()
        .add_system(
            snake_movement_input
                .label(SnakeMovement::Input)
                .before(SnakeMovement::Movement),
        )
        .add_system(game_over.after(SnakeMovement::Movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.15))
                .with_system(snake_movement.label(SnakeMovement::Movement))
                .with_system(
                    snake_eating
                        .label(SnakeMovement::Eating)
                        .after(SnakeMovement::Movement),
                )
                .with_system(
                    snake_growth
                        .label(SnakeMovement::Growth)
                        .after(SnakeMovement::Eating),
                )
                .with_system(food_spawner.after(SnakeMovement::Eating)),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .run();
}

fn food_spawner(
    mut commands: Commands,
    mut growth_reader: EventReader<GrowthEvent>,
    food: Query<Entity, With<Food>>,
    segments: Query<&Position, With<SnakeSegment>>,
) {
    if growth_reader.iter().next().is_some() || food.is_empty() {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: FOOD_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Food)
            .insert(get_available_position(segments))
            .insert(Size::square(0.8));
    }
}

fn get_available_position(segments: Query<&Position, With<SnakeSegment>>) -> Position {
    loop {
        let position = Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        };
        if !segments.iter().any(|segment_position| {
            segment_position.x == position.x && segment_position.y == position.y
        }) {
            return position;
        }
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if reader.iter().next().is_some() {
        for entity in food.iter().chain(segments.iter()) {
            commands.entity(entity).despawn();
        }
        snake_spawner(commands, segments_res);
    }
}

fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    match windows.get_primary() {
        Some(window) => {
            for (position, mut transform) in query.iter_mut() {
                transform.translation = Vec3::new(
                    convert(position.x as f32, window.width(), ARENA_WIDTH as f32),
                    convert(position.y as f32, window.height(), ARENA_HEIGHT as f32),
                    0.,
                )
            }
        }
        None => {}
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

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.iter().next().is_some() {
        segments
            .0
            .push(snake_segment_spawn(commands, last_tail_position.0.unwrap()));
    }
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
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
            Direction::None => {}
        };
        if head_position.x < 0
            || head_position.y < 0
            || head_position.x as u32 >= ARENA_WIDTH
            || head_position.y as u32 >= ARENA_HEIGHT
            || segment_positions.contains(&head_position)
        {
            game_over_writer.send(GameOverEvent);
        }
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(position, segment)| {
                *positions.get_mut(*segment).unwrap() = *position;
            });
        last_tail_position.0 = Some(*segment_positions.last().unwrap());
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
            } else if keyboard_input.pressed(KeyCode::Escape) {
                process::exit(1)
            } else {
                head.direction
            };
        if direction != head.direction.opposite() {
            head.direction = direction;
        }
    }
}

fn snake_spawner(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
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
                direction: Direction::None,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
        snake_segment_spawn(commands, Position { x: 3, y: 2 }),
    ];
}

fn snake_segment_spawn(mut commands: Commands, position: Position) -> Entity {
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
