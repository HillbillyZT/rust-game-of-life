#![allow(unused, dead_code)]

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use bevy::prelude::*;
use crate::SystemLabels::{CalcDie, CalcLive};

const GRID_VISIBLE_WIDTH: u32 = 40;
const GRID_VISIBLE_HEIGHT: u32 = 30;

struct Materials {
    living: Handle<ColorMaterial>,
    dead: Handle<ColorMaterial>
}

struct SizeScale(f32);

struct LivingCell {
    pos: Vec2,
    next_state: NextState
}

#[derive(Eq, PartialEq)]
enum NextState {
    Live,
    Die
}

struct GlobalCellList(Vec<Vec2>);

struct SpawnEvent(Vec2);

#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
enum SystemLabels {
    CalcLive,
    CalcDie,
    Tick
}


fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Conway's Game of Life".to_string(),
            width: 800.,
            height: 600.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_stage("spawn_board", SystemStage::single(spawn_board.system()))
        .add_system(check_die.system().label(SystemLabels::CalcDie))
        .add_system(check_spawn.system().label(SystemLabels::CalcLive))
        .add_system(tick_sim.system().label(SystemLabels::Tick).after(SystemLabels::CalcLive).after(SystemLabels::CalcDie))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(grid_to_screen_size.system())
                .with_system(grid_to_screen_pos.system()),
        )
        .add_event::<SpawnEvent>()
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
)
{
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        living: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        dead: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
    });
    commands.insert_resource(GlobalCellList(Vec::default()))
}

fn spawn_board(
    mut commands: Commands,
    materials: Res<Materials>,
    mut global_list: ResMut<GlobalCellList>
)
{
    //Default Living Cells:
    //Horizontal line 3-wide
    let default_mat= vec![(-1,0), (0,0), (1,0)];
    for item in default_mat {
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.living.clone(),
                ..Default::default()
            })
            .insert(LivingCell {
                pos: Vec2::new(item.0 as f32, item.1 as f32),
                next_state: NextState::Live
            })
            .insert(SizeScale(0.8)
            );
        global_list.0.push(Vec2::new(item.0 as f32, item.1 as f32));
    }
}

fn check_die(
    mut q: Query<&mut LivingCell>,
    global_cell_list: Res<GlobalCellList>
)
{
    
    //let all_cells: Vec<&LivingCell> = q.iter().collect();
    
    for (mut cell) in q.iter_mut() {
        let cell_pos: Vec2 = cell.pos;
        let neighbors = get_living_neighbor_count(&cell_pos, &global_cell_list.0);
        if neighbors > 3 || neighbors < 2 {
            cell.next_state = NextState::Die;
        }
        else {
            cell.next_state = NextState::Live;
        }
        let live = if cell.next_state == NextState::Live {
            "live"
        } else {
            "die"
        };
        println!("The cell at {0},{1} will {2}.", cell_pos.x, cell_pos.y, live);
    }
    
}

fn check_spawn(
    q: Query<&LivingCell>,
    cell_list: Res<GlobalCellList>,
    mut writer: EventWriter<SpawnEvent>
)
{
    for cell in q.iter() {
        let neighbors = get_neighboring_cells(&cell.pos);
        for n in neighbors {
            println!("Checking the cell at {0},{1} for spawn criteria.", n.x, n.y);
            if get_living_neighbor_count(&n, &cell_list.0) == 3 {
                writer.send(SpawnEvent(n));
                println!("A new cell should spawn at {0}, {1}.", n.x, n.y);
            }
        }
    }
}

fn tick_sim(
    mut commands: Commands,
    materials: Res<Materials>,
    q: Query<(Entity, &LivingCell)>,
    mut spawn_reader: EventReader<SpawnEvent>,
    mut global_list: ResMut<GlobalCellList>
)
{
    for ev in spawn_reader.iter() {
        if !global_list.0.contains(&ev.0) {
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.living.clone(),
                    ..Default::default()
                })
                .insert(LivingCell {
                    pos: ev.0,
                    next_state: NextState::Live
                })
                .insert(SizeScale(0.8)
                );
            global_list.0.push(ev.0);
            println!("Spawning cell at {0},{1}.", ev.0.x, ev.0.y);
        }
    }
    //pretty inefficient...
    for (entity, cell) in q.iter() {
        if cell.next_state == NextState::Die {
            commands.entity(entity).despawn();
            println!("Killing cell at {0},{1}.", cell.pos.x, cell.pos.y);
            global_list.0.retain(|&x| x != cell.pos);
        }
    }
    
}

fn get_living_neighbor_count(&pos: &Vec2, cells: &Vec<Vec2>) -> i32 {
    let neighbors = get_neighboring_cells(&pos);
    
    let mut n_count = 0;
    
    for n in neighbors {
        if cells.contains(&n) {
            n_count += 1;
        }
    }
    return n_count;
}

fn get_neighboring_cells(&pos: &Vec2) -> Vec<Vec2> {
    let mut neighbors_vec = Vec::default();
    let neighbors = vec![
        (-1, 1), (0, 1), (1,1),
        (-1, 0),         (1, 0),
        (-1, -1), (0, -1), (1, -1)
    ];
    
    for n in neighbors {
        let neighbor = Vec2::new(pos.x + n.0 as f32, pos.y + n.1 as f32);
        neighbors_vec.push(neighbor);
    }
    return neighbors_vec;
}


fn grid_to_screen_size(
    window: Res<Windows>, mut q: Query<(&SizeScale, &mut Sprite)>,
)
{
    let window = window.get_primary().unwrap();
    for (size_scale, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            size_scale.0 / GRID_VISIBLE_WIDTH as f32 * window.width() as f32,
            size_scale.0 / GRID_VISIBLE_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn grid_to_screen_pos(
    windows: Res<Windows>, mut q: Query<(&LivingCell, &mut Transform)>,
)
{
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        return pos / bound_game * bound_window + (tile_size / 2.);
    }
    let window = windows.get_primary().unwrap();
    for (cell, mut transform) in q.iter_mut() {
        let pos: Vec2 = cell.pos;
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, GRID_VISIBLE_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, GRID_VISIBLE_HEIGHT as f32),
            0.0,
        );
    }
}