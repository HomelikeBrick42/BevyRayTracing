use bevy::{
    app::{App, Startup, Update},
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res},
    },
    time::{Time, TimePlugin},
};
use game::{
    math::{Motor, Vector3},
    render::{Camera, MainCamera, Material, Sphere},
    transform::Transform,
    GamePlugins,
};

fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_plugins(TimePlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, spiral_spheres)
        .run()
}

#[derive(Component)]
struct SpiralMove;

fn startup(mut commands: Commands) {
    commands.spawn((
        Transform {
            motor: Motor::translation(Vector3 {
                x: -3.0,
                y: 0.0,
                z: 0.0,
            }),
        },
        Camera {
            v_fov: 90.0,
            min_distance: 0.01,
            max_distance: 100.0,
            max_bounces: 4,
        },
        MainCamera,
    ));

    commands.spawn((
        Transform {
            motor: Motor::IDENTITY,
        },
        Sphere { radius: 1.0 },
        Material {
            color: Vector3 {
                x: 0.1,
                y: 0.8,
                z: 0.2,
            },
        },
        SpiralMove,
    ));
}

fn spiral_spheres(
    mut spheres: Query<&mut Transform, (With<Sphere>, With<SpiralMove>)>,
    time: Res<Time>,
) {
    spheres.for_each_mut(|mut sphere| {
        let time = time.elapsed_seconds() * 2.0;
        sphere.motor = Motor::translation(Vector3 {
            x: time.sin(),
            y: (time * 0.33).cos() * 2.0,
            z: time.cos(),
        });
    });
}
