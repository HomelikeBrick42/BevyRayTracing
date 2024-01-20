use crate::{
    math::{Motor, Vector3},
    render::{Camera, MainCamera, Material, Sphere},
    transform::GlobalTransform,
    window::InitWindowResource,
};
use bevy::ecs::{
    change_detection::DetectChanges,
    system::{Query, Res, ResMut, Resource},
    world::{FromWorld, Ref, World},
};
use encase::{ArrayLength, ShaderSize, ShaderType, StorageBuffer, UniformBuffer};
use std::sync::Arc;
use winit::window::Window;

#[derive(ShaderType)]
struct GpuCamera {
    transform: Motor,
    v_fov: f32,
    min_distance: f32,
    max_distance: f32,
    sun_direction: Vector3,
}

#[derive(ShaderType)]
struct GpuSphere {
    transform: Motor,
    color: Vector3,
    radius: f32,
}

#[derive(ShaderType)]
struct GpuSpheres<'a> {
    length: ArrayLength,
    #[size(runtime)]
    data: &'a [GpuSphere],
}

#[derive(Resource)]
pub(super) struct RenderState {
    ray_tracing_pipeline: wgpu::ComputePipeline,

    sphere_bind_group_layout: wgpu::BindGroupLayout,

    camera_bind_group: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,

    main_texture_bind_group: wgpu::BindGroup,
    main_texture_bind_group_layout: wgpu::BindGroupLayout,
    main_texture: wgpu::Texture,

    queue: wgpu::Queue,
    device: wgpu::Device,

    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface,

    // we must keep the window alive so it is destructed after the surface
    window: Arc<Window>,
}

impl FromWorld for RenderState {
    fn from_world(world: &mut World) -> Self {
        let window = world
            .get_non_send_resource::<InitWindowResource>()
            .unwrap()
            .main_window
            .clone();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None,
                )
                .await
                .unwrap();

            (adapter, device, queue)
        });

        let size = window.inner_size();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::COPY_DST,
            format: surface_capabilities
                .formats
                .iter()
                .filter(|format| {
                    matches!(format.remove_srgb_suffix(), wgpu::TextureFormat::Rgba8Unorm)
                })
                .max_by_key(|format| format.is_srgb())
                .copied()
                .expect("surface should support some kind of rgba8unorm format"),
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: surface_capabilities
                .alpha_modes
                .iter()
                .find(|alpha_mode| matches!(alpha_mode, wgpu::CompositeAlphaMode::Opaque))
                .copied()
                .unwrap_or(surface_capabilities.alpha_modes[0]),
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let main_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Main Texture"),
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let main_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Main Texture Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let main_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Texture Bind Group"),
            layout: &main_texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &main_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: GpuCamera::SHADER_SIZE.get(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuCamera::SHADER_SIZE),
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        let sphere_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sphere Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuSpheres::<'_>::min_size()),
                    },
                    count: None,
                }],
            });

        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &main_texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &sphere_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let ray_tracing_shader =
            device.create_shader_module(wgpu::include_wgsl!("./ray_tracing.wgsl"));
        let ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                module: &ray_tracing_shader,
                entry_point: "ray_trace",
            });

        RenderState {
            ray_tracing_pipeline,

            sphere_bind_group_layout,

            camera_bind_group,
            camera_uniform_buffer,

            main_texture_bind_group,
            main_texture_bind_group_layout,
            main_texture,

            queue,
            device,

            surface_config,
            surface,

            window,
        }
    }
}

impl RenderState {
    fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);

        self.main_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Main Texture"),
            size: wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        self.main_texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Texture Bind Group"),
            layout: &self.main_texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &self
                        .main_texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });
    }
}

#[derive(Resource)]
pub(super) struct SphereState {
    sphere_buffer: wgpu::Buffer,
    sphere_bind_group: wgpu::BindGroup,
    spheres: Vec<GpuSphere>,
    buffer: Vec<u8>,
}

impl FromWorld for SphereState {
    fn from_world(world: &mut World) -> Self {
        let render_state = world.get_resource_mut::<RenderState>().unwrap();

        let sphere_buffer = render_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sphere Buffer"),
            size: GpuSpheres::<'_>::min_size().get(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let sphere_bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Sphere Bind Group"),
                layout: &render_state.sphere_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sphere_buffer.as_entire_binding(),
                }],
            });

        SphereState {
            sphere_buffer,
            sphere_bind_group,
            spheres: vec![],
            buffer: vec![],
        }
    }
}

pub(super) fn update_spheres(
    render_state: Res<RenderState>,
    mut sphere_state: ResMut<SphereState>,
    spheres: Query<(Ref<GlobalTransform>, Ref<Material>, Ref<Sphere>)>,
) {
    let sphere_state: &mut SphereState = &mut sphere_state;

    let previous_sphere_count = sphere_state.spheres.len();
    sphere_state.buffer.clear();

    let mut components_changed = false;
    sphere_state.spheres.clear();
    spheres.for_each(|(transform, material, sphere)| {
        components_changed |=
            transform.is_changed() || material.is_changed() || sphere.is_changed();
        let Material { color } = *material;
        let Sphere { radius } = *sphere;
        sphere_state.spheres.push(GpuSphere {
            transform: transform.transform().motor,
            color,
            radius,
        });
    });

    if components_changed || sphere_state.spheres.len() != previous_sphere_count {
        let mut buffer = StorageBuffer::new(&mut sphere_state.buffer);
        buffer
            .write(&GpuSpheres {
                length: ArrayLength,
                data: &sphere_state.spheres,
            })
            .unwrap();

        if sphere_state.buffer.len() as wgpu::BufferAddress > sphere_state.sphere_buffer.size() {
            sphere_state.sphere_buffer =
                render_state.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Sphere Buffer"),
                    size: sphere_state.buffer.len() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                });

            sphere_state.sphere_bind_group =
                render_state
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Sphere Bind Group"),
                        layout: &render_state.sphere_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: sphere_state.sphere_buffer.as_entire_binding(),
                        }],
                    });
        }

        render_state
            .queue
            .write_buffer(&sphere_state.sphere_buffer, 0, &sphere_state.buffer);
    }
}

pub(super) fn update_camera(
    render_state: Res<RenderState>,
    camera: Query<(Ref<GlobalTransform>, Ref<Camera>, Ref<MainCamera>)>,
) {
    let (global_transform, camera, main_camera) = camera.single();
    if global_transform.is_changed() || camera.is_changed() || main_camera.is_changed() {
        let mut buffer = UniformBuffer::new([0; GpuCamera::SHADER_SIZE.get() as _]);
        let Camera {
            v_fov,
            min_distance,
            max_distance,
            sun_direction,
        } = *camera;
        buffer
            .write(&GpuCamera {
                transform: global_transform.transform().motor,
                v_fov,
                min_distance,
                max_distance,
                sun_direction,
            })
            .unwrap();
        render_state.queue.write_buffer(
            &render_state.camera_uniform_buffer,
            0,
            &buffer.into_inner(),
        );
    }
}

pub(super) fn render(mut render_state: ResMut<RenderState>, sphere_state: Res<SphereState>) {
    let output = loop {
        match render_state.surface.get_current_texture() {
            Ok(output) => break output,
            Err(error) => match error {
                e @ wgpu::SurfaceError::Timeout => {
                    eprintln!("{e}");
                    return;
                }

                wgpu::SurfaceError::Outdated => {
                    let size = render_state.window.inner_size();
                    render_state.resize(size.width, size.height);
                }

                wgpu::SurfaceError::Lost => {
                    render_state
                        .surface
                        .configure(&render_state.device, &render_state.surface_config);
                }

                e @ wgpu::SurfaceError::OutOfMemory => panic!("{e}"),
            },
        }
    };

    let mut encoder = render_state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
    {
        let mut ray_tracing_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Tracing Pass"),
            timestamp_writes: None,
        });

        ray_tracing_pass.set_pipeline(&render_state.ray_tracing_pipeline);
        ray_tracing_pass.set_bind_group(0, &render_state.main_texture_bind_group, &[]);
        ray_tracing_pass.set_bind_group(1, &render_state.camera_bind_group, &[]);
        ray_tracing_pass.set_bind_group(2, &sphere_state.sphere_bind_group, &[]);
        ray_tracing_pass.dispatch_workgroups(
            (render_state.main_texture.width() + (16 - 1)) / 16,
            (render_state.main_texture.height() + (16 - 1)) / 16,
            1,
        );
    }
    encoder.copy_texture_to_texture(
        render_state.main_texture.as_image_copy(),
        output.texture.as_image_copy(),
        wgpu::Extent3d {
            width: render_state.surface_config.width,
            height: render_state.surface_config.height,
            depth_or_array_layers: 1,
        },
    );
    render_state.queue.submit([encoder.finish()]);

    render_state.window.pre_present_notify();
    output.present();
}
