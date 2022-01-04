#![allow(unused, dead_code)]

mod game;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use bevy::app::Events;
use bevy::core::FixedTimestep;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

const GRID_VISIBLE_WIDTH: u32 = 40;
const GRID_VISIBLE_HEIGHT: u32 = 30;

struct Materials {
    living: Handle<ColorMaterial>,
    dead: Handle<ColorMaterial>,
    axis: Handle<ColorMaterial>
}

struct ViewOffset(Vec2);

struct SizeScale(f32);


fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Conway's Game of Life".to_string(),
            width: 800.,
            height: 600.,
            ..Default::default()
        })
        .insert_resource(ViewOffset(Vec2::default()))
        .add_plugins(DefaultPlugins)
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(game::GamePlugin)
        .add_startup_system(setup.system().label("setup"))
        //.add_startup_system(draw_axis.system().label("axis").after("setup"))
        .add_startup_stage("draw_axis", SystemStage::single(draw_axis.system()))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                //.with_run_criteria(FixedTimestep::step(0.4))
                .with_system(grid_to_screen_size.system())
                .with_system(grid_to_screen_pos.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.02))
                .with_system(update_offset.system())
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>
)
{
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        living: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        dead: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
        axis: materials.add(Color::rgba(1.,0.,0., 1.).into())
    });
}

fn draw_axis(
    mut commands: Commands,
    windows: Res<Windows>,
    mut materials: ResMut<Materials>
)
{
    let window = windows.get_primary().unwrap();
    let height = window.height();
    let width = window.width();
    commands.spawn_bundle(SpriteBundle {
        material: materials.axis.clone(),
        sprite: Sprite::new(Vec2::new(2.0, height as f32)),
        transform: Transform::from_xyz(0.,0.,0.),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        material: materials.axis.clone(),
        sprite: Sprite::new(Vec2::new(width as f32, 2.0)),
        transform: Transform::from_xyz(0.,0.,0.),
        ..Default::default()
    });
}

fn update_offset(
    kb_input: Res<Input<KeyCode>>,
    mut offset: ResMut<ViewOffset>
) {
    if kb_input.pressed(KeyCode::W){
        offset.0.y += 1.;
    }
    if kb_input.pressed(KeyCode::A) {
        offset.0.x -= 1.;
    }
    if kb_input.pressed(KeyCode::S) {
        offset.0.y -= 1.;
    }
    if kb_input.pressed(KeyCode::D) {
        offset.0.x += 1.;
    }
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
    windows: Res<Windows>,
    mut q: Query<(&game::LivingCell, &mut Transform)>,
    offset: Res<ViewOffset>
)
{
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        return pos / bound_game * bound_window /*+ (tile_size / 2.)*/;
    }
    let window = windows.get_primary().unwrap();
    for (cell, mut transform) in q.iter_mut() {
        let pos: Vec2 = cell.pos - offset.0;
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, GRID_VISIBLE_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, GRID_VISIBLE_HEIGHT as f32),
            0.0,
        );
    }
}



