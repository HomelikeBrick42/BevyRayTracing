mod render_state;

use crate::{
    math::Vector3,
    render::render_state::{RenderState, SphereState},
};
use bevy::{
    app::{App, Plugin},
    ecs::{
        component::Component,
        schedule::{IntoSystemConfigs, Schedule, ScheduleLabel},
    },
};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderState>()
            .init_resource::<SphereState>();

        let mut render_schedule = Schedule::new(RenderSchedule);
        render_schedule.add_systems(
            (
                (render_state::update_camera, render_state::update_spheres),
                render_state::render,
            )
                .chain(),
        );
        app.world.add_schedule(render_schedule);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ScheduleLabel)]
pub struct RenderSchedule;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Camera {
    pub v_fov: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub max_bounces: u32,
}

#[derive(Component)]
pub struct Material {
    pub color: Vector3,
}

#[derive(Component)]
pub struct Sphere {
    pub radius: f32,
}
