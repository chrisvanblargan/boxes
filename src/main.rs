use std::{
    cmp::Ordering,
    collections::HashMap,
    convert::{TryFrom, TryInto},
    ops::Range,
};

use bevy::prelude::*;
use bevy_easings::*;
use itertools::Itertools;
use rand::prelude::*;

mod ui;
use ui::*;

mod colors;
use colors::*;

const TILE_SIZE: f32 = 40.0;
const TILE_SPACER: f32 = 10.0;

#[derive(Component)]
struct Board {
    size: u8,
    physical_size: f32,
}

#[derive(Debug, PartialEq, Component)]
struct Points {
    value: u32,
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash, Component,)]
struct Position {
    x: u8,
    y: u8,
}

#[derive(Component)]
struct TileText;

struct FontSpec {
    family: Handle<Font>,
}

impl FromWorld for FontSpec {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world
            .get_resource_mut::<AssetServer>()
            .unwrap();
        FontSpec {
            family: asset_server
                .load("fonts/FiraSans-Bold.ttf"),
        }
    }
}

impl Board {
    fn new(size: u8) -> Self {
        let physical_size = f32::from(size) * TILE_SIZE
            + f32::from(size + 1) * TILE_SPACER;
        Board {
            size,
            physical_size,
        }
    }
    fn cell_position_to_physical(&self, pos: u8) -> f32 {
        let offset =
            -self.physical_size / 2.0 + 0.5 * TILE_SIZE;

        offset
            + f32::from(pos) * TILE_SIZE
            + f32::from(pos + 1) * TILE_SPACER
    }
}

enum BoardShift {
    Left,
    Right,
    Up,
    Down,
}

impl TryFrom<&KeyCode> for BoardShift {
    type Error = &'static str;

    fn try_from(value: &KeyCode) -> Result<Self, Self::Error> {
        match value {
            KeyCode::Left => Ok(BoardShift::Left),
            KeyCode::Right => Ok(BoardShift::Right),
            KeyCode::Up => Ok(BoardShift::Up),
            KeyCode::Down => Ok(BoardShift::Down),
            _ => Err("not a valid board shift key")
        }
    }
}

impl BoardShift {
    fn sort(&self, a: &Position, b: &Position) -> Ordering {
        match self {
            BoardShift::Left => {
                match Ord::cmp(&a.y, &b.y) {
                    Ordering::Equal => { 
                        Ord::cmp(&a.x, &b.x)
                    }
                    ordering => ordering,
                }
            },
            BoardShift::Right => {
                match Ord::cmp(&b.y, &a.y) {
                    Ordering::Equal => { 
                        Ord::cmp(&b.x, &a.x)
                    }
                    a => a,
                }
            },            
            BoardShift::Up => match Ord::cmp(&b.x, &a.x) {
                Ordering::Equal => { 
                    Ord::cmp(&b.y, &a.y)
                }
                ordering => ordering,
            },
            BoardShift::Down => match Ord::cmp(&a.x, &b.x) {
                Ordering::Equal => { 
                    Ord::cmp(&a.y, &b.y)
                }
                ordering => ordering,
            },
        }
    }
    fn set_column_position(
        &self,
        board_size: u8,
        position: &mut Mut<Position>,
        index: u8,
    ) {
        match self {
            BoardShift::Left => {
                position.x = index;
                //dbg!("left");
            },
            BoardShift::Right => {
                position.x = board_size - 1 - index;
                //dbg!("right");
            },            
            BoardShift::Up => {
                position.y = board_size - 1 - index;
                //dbg!("up");
            },
            BoardShift::Down => {
                position.y = index;
                //dbg!("down");
            },
        }
    }
    fn get_row_position(
        &self,
        position: &Position,
    ) -> u8 {
        match self {
            BoardShift::Left => position.y,
            BoardShift::Right => position.y,            
            BoardShift::Up => position.x,
            BoardShift::Down => position.x,
        }
    }
}

struct NewTileEvent;

#[derive(Default)]
struct Game {
    score: u32,
    score_best: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum RunState {
    Playing,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GameUiPlugin)
        .add_plugin(EasingsPlugin)
        .init_resource::<FontSpec>()
        .init_resource::<Game>()
        .add_event::<NewTileEvent>()
        .add_state(RunState::Playing)
        .add_startup_system(setup)
        .add_startup_system(spawn_board)
        .add_system_set(
            SystemSet::on_update(RunState::Playing)
            .with_system(render_tile_points)
            .with_system(board_shift)
            .with_system(render_tiles)
            .with_system(new_tile_handler)
            .with_system(end_game),
        )
        .add_system_set(
            SystemSet::on_enter(RunState::Playing)
                .with_system(game_reset)
                .with_system(spawn_tiles),
        )
        .run()
    
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default());
}

fn spawn_board (mut commands: Commands) {
    let board = Board::new(4);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: MATERIALS.board,
                custom_size: Some(Vec2::new(
                    board.physical_size,
                    board.physical_size,
                )),
                ..Sprite::default()
            },
            ..Default::default()
        })
        .with_children(|builder| {
            for tile in (0..board.size)
                .cartesian_product(0..board.size) //creates tuples of board coordinates to position tiles (0,0)..(3,3)
            {
                builder.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: MATERIALS.tile_placeholder,
                        custom_size: Some(Vec2::new(
                            TILE_SIZE,
                            TILE_SIZE,
                        )),
                        ..Sprite::default()
                    },
                    transform: Transform::from_xyz(
                        board.cell_position_to_physical(
                            tile.0,
                        ),
                        board.cell_position_to_physical(
                            tile.1,
                        ),
                        1.0,
                    ),
                    ..Default::default()
                });
            }
        })
        .insert(board);
}

fn spawn_tiles(
    mut commands: Commands,
    query_board: Query<&Board>,
    font_spec: Res<FontSpec>,
) {
    let board = query_board.single();
    let mut rng = rand::thread_rng();
    let starting_tiles: Vec<(u8, u8)> = (0..board.size)
        .cartesian_product(0..board.size)
        .choose_multiple(&mut rng, 2);
    for (x, y) in starting_tiles.iter() {
        let pos = Position { x: *x, y: *y};
        spawn_tile(&mut commands, board, &font_spec, pos);
    }
}
        
fn spawn_tile(
    commands: &mut Commands,
    board: &Board,
    font_spec: &Res<FontSpec>,
    pos: Position,
) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: MATERIALS.tile,
                custom_size: Some(Vec2::new(
                    TILE_SIZE, TILE_SIZE,
                )),
                ..Sprite::default()
            },
            transform: Transform::from_xyz(
                board.cell_position_to_physical(pos.x),
                board.cell_position_to_physical(pos.y),
                2.0,
            ),
            ..Default::default()
        })
        .with_children(|child_builder| {
            child_builder
                .spawn_bundle(Text2dBundle {
                    text: Text::from_section(
                        "2", 
                        TextStyle { 
                            font: font_spec.family.clone(), 
                            font_size: 40.0, 
                            color: Color::BLACK, 
                            ..Default::default()
                        },
                    )
                    .with_alignment(TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    }),
                    transform: Transform::from_xyz(
                        0.0, 0.0, 1.0,
                    ),
                    ..Default::default()
                })
                .insert(TileText);
        })
        .insert(Points { value : 2 })
        .insert(pos);
}

fn render_tile_points(
    mut texts: Query<&mut Text, With<TileText>>,
    tiles: Query<(&Points, &Children)>,
){
    for (points, children) in tiles.iter() {
        if let Some(entity) = children.first() {
            let mut text = texts
                .get_mut(*entity)
                .expect("expected Text to exist");
            let mut text_section = text.sections.first_mut().expect("expect first section to be accessible as mutable");
            text_section.value = points.value.to_string()
        }
    }

}

fn board_shift (
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut tiles: Query<(Entity, &mut Position, &mut Points)>,
    query_board: Query<&Board>,
    mut tile_writer: EventWriter<NewTileEvent>,
    mut game: ResMut<Game>,
) {
    let board = query_board.single();
    let shift_direction =
        keyboard_input.get_just_pressed().find_map(
            |key_code| BoardShift::try_from(key_code).ok(),
        );

    if let Some(board_shift) = shift_direction {
        let mut it =
            tiles.iter_mut().sorted_by(|a, b| board_shift.sort(&a.1, &b.1))
            .peekable();

        let mut column: u8 = 0;
        
        while let Some(mut tile) = it.next() {
            board_shift.set_column_position(
                board.size,
                &mut tile.1,
                column,
            );
            if let Some(tile_next) = it.peek() {
                    if board_shift.get_row_position(&tile.1) != board_shift.get_row_position(&tile_next.1) {
                        //different row, no merge
                        column = 0;
                    } else if tile.2.value != tile_next.2.value {
                        //different value, no merge
                        column = column + 1;
                    } else{
                        //merge
                    let real_next_tile = it 
                        .next()
                        .expect("a peeked tile should always exist when we .next");
                    tile.2.value = tile.2.value + real_next_tile.2.value;
                    game.score += tile.2.value;

                    if game.score_best < game.score {
                        game.score_best = game.score;
                    }

                    commands
                        .entity(real_next_tile.0)
                        .despawn_recursive();

                    if let Some(future) = it.peek() {
                        if board_shift.get_row_position(&tile.1) != board_shift.get_row_position(&future.1) {
                            column = 0;
                        } else {
                            column = column + 1;
                        }
                    }
                }
            }
        }
        //dbg!(game.score);
        tile_writer.send(NewTileEvent);
    }
}

fn render_tiles(
    mut commands: Commands,
    mut tiles: Query<
        (Entity, &mut Transform, &Position),
        Changed<Position>,
    >,
    query_board: Query<&Board>,
) {
    let board = query_board.single();

     for (entity, mut transform, pos) in tiles.iter_mut() {
            let x = board.cell_position_to_physical(pos.x);
            let y = board.cell_position_to_physical(pos.y);

            commands.entity(entity).insert(transform.ease_to(
                Transform::from_xyz(
                    x,
                    y,
                    transform.translation.z,
                ),
                EaseFunction::QuadraticInOut,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(
                        100,
                    ),
                },
            ));
    }
}

fn new_tile_handler(
    mut tile_reader: EventReader<NewTileEvent>,
    mut commands: Commands,
    query_board: Query<&Board>,
    tiles: Query<&Position>,
    font_spec: Res<FontSpec>
) {
    let board = query_board.single();

    for _event in tile_reader.iter() {
        //insert new tile
        let mut rng = rand::thread_rng();
        let possible_position: Option<Position> = (
            0..board.size)
            .cartesian_product(0..board.size)
            .filter_map(|tile_pos| {
                let new_pos = Position {
                    x: tile_pos.0,
                    y: tile_pos.1,
                };
                match tiles
                    .iter()
                    .find(|&&pos| pos == new_pos)
                {
                    Some(_) => None,
                    None => Some(new_pos),
                }
            })
            .choose(&mut rng);

        if let Some(pos) = possible_position {
            spawn_tile(
                &mut commands,
                board,
                &font_spec,
                pos,
            );
        }
    }
}

fn end_game(
    tiles: Query<(&Position, &Points)>,
    query_board: Query<&Board>,
    mut run_state: ResMut<State<RunState>>,
) {
    let board = query_board.single();

    if tiles.iter().len() == 16 {
        let map: HashMap<&Position, &Points> =
        tiles.iter().collect();

        let neighbor_points =
        [(-1, 0), (0, 1), (1, 0), (0, -1),];
        let board_range: Range<i8> = 0..(board.size as i8);

        let has_move = tiles
            .iter().any(
                |(Position {x, y}, value)| {
                neighbor_points
                    .iter()
                    .filter_map(|(x2, y2)| {
                        let new_x = *x as i8 - x2;
                        let new_y = *y as i8 -y2;

                        if !board_range.contains(&new_x)
                            || !board_range.contains(&new_y) 
                        {
                            return None;
                        };

                        map.get(&Position {
                            x: new_x.try_into().unwrap(),
                            y: new_y.try_into().unwrap(),
                        })
                    })
                    .any(|&v| v == value)
                },
            );

            if has_move == false {
                dbg!("game over");
                run_state.set(RunState::GameOver).unwrap();
            }
    };
}

fn game_reset(
    mut commands: Commands,
    tiles: Query<Entity, With<Position>>,
    mut game: ResMut<Game>,
) {
    for entity in tiles.iter() {
        commands.entity(entity).despawn_recursive();
    }
    game.score = 0;
}