use crate::math::Motor;
use bevy::{
    app::{App, Plugin, PostStartup, PostUpdate},
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        query::{With, Without},
        removal_detection::RemovedComponents,
        schedule::IntoSystemConfigs as _,
        system::{Commands, Query},
        world::Ref,
    },
    hierarchy::Parent,
};

pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        macro_rules! systems {
            () => {
                (
                    (
                        add_global_transforms,
                        remove_global_transforms,
                        update_removed_parents,
                    ),
                    update_global_transforms,
                )
                    .chain()
            };
        }

        app.add_systems(PostStartup, systems!());
        app.add_systems(PostUpdate, systems!());
    }
}

#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub motor: Motor,
}

#[derive(Component, Clone, Copy)]
pub struct GlobalTransform(Transform);

impl GlobalTransform {
    pub fn transform(&self) -> Transform {
        self.0
    }
}

fn add_global_transforms(
    mut commands: Commands,
    transforms: Query<(Entity, &Transform), Without<GlobalTransform>>,
) {
    transforms.for_each(|(entity, &transform)| {
        commands.entity(entity).insert(GlobalTransform(transform));
    });
}

fn remove_global_transforms(
    mut commands: Commands,
    global_transforms: Query<Entity, (With<GlobalTransform>, Without<Transform>)>,
) {
    global_transforms.for_each(|entity| {
        commands.entity(entity).remove::<GlobalTransform>();
    });
}

/// this exists to make sure that when a parent component is removed the child's transform gets updated correctly
fn update_removed_parents(
    mut removed_parent_components: RemovedComponents<Parent>,
    mut transform: Query<&mut Transform>,
) {
    removed_parent_components
        .read()
        .for_each(|parent_component| {
            if let Ok(mut transform) = transform.get_mut(parent_component) {
                // set changed flag
                let _: &mut Transform = &mut transform;
            }
        });
}

fn update_global_transforms(
    mut global_transforms: Query<(Entity, &mut GlobalTransform)>,
    transforms: Query<(Ref<Transform>, Option<Ref<Parent>>)>,
) {
    global_transforms
        .par_iter_mut()
        .for_each(|(entity, mut global_transform)| {
            let (mut current_transform, mut current_parent) = transforms.get(entity).unwrap();

            let mut transform_changed = current_transform.is_changed()
                || current_parent
                    .as_ref()
                    .map_or(false, |parent| parent.is_changed());

            let mut transform = *current_transform;

            while let Some(parent) = current_parent {
                (current_transform, current_parent) = transforms.get(parent.get()).unwrap();

                transform_changed |= current_transform.is_changed()
                    || current_parent
                        .as_ref()
                        .map_or(false, |parent| parent.is_changed());

                transform.motor = transform.motor.pre_apply(current_transform.motor);
            }

            if transform_changed {
                *global_transform = GlobalTransform(transform);
            }
        });
}
