use std::{
    cmp::Ordering,
    collections::HashMap,
    convert::{TryFrom, TryInto},
    ops::Range,
};

use bevy::{prelude::*, sprite};
use itertools::Itertools;
use rand::prelude::*;

//mod ui;
//use ui::*;
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

#[derive(
    Debug, PartialEq, Copy, Clone, Eq, Hash, Component,
)]
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<FontSpec>()
        .add_startup_system(setup)
        .add_startup_system(spawn_board)
        .add_startup_system_to_stage(
            StartupStage::PostStartup,
            spawn_tiles,
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
                            color: Color::VIOLET, 
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
