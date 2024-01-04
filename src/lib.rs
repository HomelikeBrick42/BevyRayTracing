pub mod math;
pub mod render;
pub mod transform;
pub mod window;

use bevy::app::{PluginGroup, PluginGroupBuilder};
use render::RenderPlugin;
use transform::TransformPlugin;
use window::WindowPlugin;

pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(WindowPlugin)
            .add_after::<WindowPlugin, _>(RenderPlugin)
            .add(TransformPlugin)
    }
}
