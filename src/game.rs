use bevy::app::Events;
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use crate::{Materials, SizeScale};

pub struct GamePlugin;

const TPS: f32 = 20.;
const TPS_STEP: f32 = 1. / TPS;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<Events<SpawnEvent>>()
            .add_startup_stage("spawn_board", SystemStage::single(spawn_board.system()))
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(TPS_STEP as f64))
                    .with_system(check_die
                        .system()
                        .label(SystemLabels::CalcDie))
                    .with_system(check_spawn
                        .system()
                        .label(SystemLabels::CalcLive))
                    .with_system(tick_sim
                        .system()
                        .label(SystemLabels::Tick)
                        .after(SystemLabels::CalcLive)
                        .after(SystemLabels::CalcDie).
                        before(SystemLabels::Manage))
                    .with_system(my_event_manager
                        .system()
                        .label(SystemLabels::Manage)
                        .after(SystemLabels::Tick))
            )
        .insert_resource(GlobalCellList(Vec::default()))
        ;
        
    }
}

pub struct LivingCell {
    pub pos: Vec2,
    pub next_state: NextState
}

#[derive(Eq, PartialEq)]
pub enum NextState {
    Live,
    Die
}

pub struct GlobalCellList(Vec<Vec2>);

struct SpawnEvent(Vec2);

#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
enum SystemLabels {
    CalcLive,
    CalcDie,
    Tick,
    Manage
}

fn spawn_board(
    mut commands: Commands,
    materials: Res<Materials>,
    mut global_list: ResMut<GlobalCellList>
)
{
    //Default Living Cells:
    //Horizontal line 3-wide
    
    //Min Stable:
    //let default_mat= vec![(-1,0), (0,0), (1,0)];
    
    //Glider:
    let default_mat = vec![(1,0), (0,1), (0,2), (1,2), (2,2)];
    
    for item in default_mat {
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.living.clone(),
                transform: Transform::from_xyz(0.,0.,0.5),
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
        //println!("The cell at {0},{1} will {2}.", cell_pos.x, cell_pos.y, live);
    }
    
}

fn check_spawn(
    q: Query<&LivingCell>,
    cell_list: Res<GlobalCellList>,
    mut writer: EventWriter<SpawnEvent>
)
{
    //println!("SPAWN CHECK RUNNING");
    for cell in q.iter() {
        let neighbors = get_neighboring_cells(&cell.pos);
        for n in neighbors {
            //println!("Checking the cell at {0},{1} for spawn criteria.", n.x, n.y);
            if get_living_neighbor_count(&n, &cell_list.0) == 3 {
                writer.send(SpawnEvent(n));
                //println!("A new cell should spawn at {0}, {1}.", n.x, n.y);
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
    // println!("Global list at beginning of Tick:");
    // for item in &global_list.0 {
    //     print!("{} ", item);
    // }
    // println!();
    
    for ev in spawn_reader.iter() {
        //println!("SHOULD SPAWN {0}", ev.0);
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
        (-1, 1), (0, 1), (1, 1),
        (-1, 0),         (1, 0),
        (-1,-1), (0,-1), (1,-1)
    ];
    
    for n in neighbors {
        let neighbor = Vec2::new(pos.x + n.0 as f32, pos.y + n.1 as f32);
        neighbors_vec.push(neighbor);
    }
    return neighbors_vec;
}

fn my_event_manager(
    mut e: ResMut<Events<SpawnEvent>>
)
{
    e.update();
}