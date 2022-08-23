use bevy::prelude::*;

const TILE_SIZE: f32 = 40.0;
const TILE_SPACER: f32 = 10.0;

#[derive(Component)]
struct Board {
    size: u8,
    physical_size: f32,
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
        .add_startup_system(setup)
        .add_startup_system(spawn_board)
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
                custom_size: Some(Vec2::new(
                    board.physical_size,
                    board.physical_size,
                )),
                ..Sprite::default()
            },
            ..Default::default()
        })
        .insert(board);
}
        
