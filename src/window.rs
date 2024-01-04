use crate::render::RenderSchedule;
use bevy::{
    app::{App, Plugin},
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
};
use std::sync::Arc;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowSize>()
            .init_non_send_resource::<InitWindowResource>()
            .set_runner(Self::runner);
    }
}

impl WindowPlugin {
    pub fn runner(mut app: App) {
        let InitWindowResource {
            main_window,
            event_loop,
        } = app.world.remove_non_send_resource().unwrap();

        main_window.set_visible(true);
        event_loop
            .run(|event, event_loop_window_target| match event {
                Event::NewEvents(StartCause::Init) => {
                    event_loop_window_target.set_control_flow(ControlFlow::Poll);
                }
                Event::AboutToWait => {
                    app.update();
                    main_window.request_redraw();
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::RedrawRequested,
                } if window_id == main_window.id() => {
                    _ = app.world.try_run_schedule(RenderSchedule);
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CloseRequested,
                } if window_id == main_window.id() => {
                    main_window.set_visible(false);
                    event_loop_window_target.exit();
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Resized(size),
                } if window_id == main_window.id() => {
                    *app.world.get_resource_mut::<WindowSize>().unwrap() = WindowSize {
                        width: size.width.max(1) as _,
                        height: size.height.max(1) as _,
                    };
                }
                _ => {}
            })
            .unwrap()
    }
}

#[derive(Resource)]
pub struct WindowSize {
    width: usize,
    height: usize,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
        }
    }
}

impl WindowSize {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

pub(crate) struct InitWindowResource {
    pub(crate) main_window: Arc<Window>,
    event_loop: EventLoop<()>,
}

impl FromWorld for InitWindowResource {
    fn from_world(_world: &mut World) -> Self {
        let event_loop = EventLoopBuilder::new().build().unwrap();
        let main_window = WindowBuilder::new()
            .with_title("Game")
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        Self {
            event_loop,
            main_window: Arc::new(main_window),
        }
    }
}
