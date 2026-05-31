use std::collections::HashMap;

use glam::Vec2;
use phantom_core::ecs::components::camera::{self, Camera};
use phantom_core::ecs::components::{Sprite, Transform};
use phantom_core::ecs::{Entity, World};
use wgpu::util::DeviceExt;

use crate::asset_manager::asset_types::texture::Texture;
use crate::renderer::vertex::Vertex;

pub struct SceneRenderer {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    gpu_textures: HashMap<String, (wgpu::BindGroup, Vec2)>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,

    /// Last camera matrix that was finite, used as a fallback so a NaN/inf in any
    /// entity's transform (e.g. a script normalizing a zero-length direction)
    /// can't collapse the projection and black out the entire scene.
    last_camera_matrix: glam::Mat4,

    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

impl SceneRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite_shader.wgsl").into()),
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camerea_buffer"),
            size: std::mem::size_of::<glam::Mat4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&texture_bind_group_layout),
                    Some(&camera_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let (depth_texture, depth_view) = Self::create_depth_texture(device, 1, 1);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()],   // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview_mask: None, // 5.
            cache: None,          // 6.
        });

        Self {
            render_pipeline,
            bind_group_layout: texture_bind_group_layout,
            gpu_textures: HashMap::new(),
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            last_camera_matrix: glam::Mat4::IDENTITY,
            depth_texture,
            depth_view,
        }
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        (texture, view)
    }

    pub fn build_sprite_verticies(&self, world: &World) -> (Vec<Vertex>, Vec<Entity>) {
        let mut entities = world.query_with2::<Sprite, Transform>();
        entities.sort_by(|a, b| {
            let za = world
                .get_component::<Transform>(*a)
                .map(|t| t.position.z)
                .unwrap_or(0.0);
            let zb = world
                .get_component::<Transform>(*b)
                .map(|t| t.position.z)
                .unwrap_or(0.0);
            za.partial_cmp(&zb).unwrap_or(std::cmp::Ordering::Equal)
        });
        let vertices = entities
            .iter()
            .flat_map(|entity| {
                let transform = world.get_component::<Transform>(*entity).unwrap();
                let sprite = world.get_component::<Sprite>(*entity).unwrap();
                let (_, _, angle) = transform.rotation.to_euler(glam::EulerRot::XYZ);
                let cos_a = angle.cos();
                let sin_a = angle.sin();

                let (tw, th) = self
                    .gpu_textures
                    .get(&sprite.asset_path)
                    .map(|(_, dims)| (dims.x, dims.y))
                    .unwrap_or((1.0, 1.0));

                let sx = transform.scale.x * tw / 2.0;
                let sy = transform.scale.y * th / 2.0;
                let px = transform.position.x;
                let py = transform.position.y;
                let pz = transform.position.z;

                let vertices: Vec<Vertex> = vec![
                    // Triangle 1
                    Vertex {
                        position: [
                            (-sx * cos_a - sy * sin_a) + px,
                            (-sx * sin_a + sy * cos_a) + py,
                            pz,
                        ],
                        tex_coords: [0.0, 0.0],
                    }, // Top Left
                    Vertex {
                        position: [
                            (-sx * cos_a + sy * sin_a) + px,
                            (-sx * sin_a - sy * cos_a) + py,
                            pz,
                        ],
                        tex_coords: [0.0, 1.0],
                    }, // Bottom Left
                    Vertex {
                        position: [
                            (sx * cos_a + sy * sin_a) + px,
                            (sx * sin_a - sy * cos_a) + py,
                            pz,
                        ],
                        tex_coords: [1.0, 1.0],
                    }, // Bottom Right
                    // Triangle 2
                    Vertex {
                        position: [
                            (-sx * cos_a - sy * sin_a) + px,
                            (-sx * sin_a + sy * cos_a) + py,
                            pz,
                        ],
                        tex_coords: [0.0, 0.0],
                    }, // Top Left
                    Vertex {
                        position: [
                            (sx * cos_a + sy * sin_a) + px,
                            (sx * sin_a - sy * cos_a) + py,
                            pz,
                        ],
                        tex_coords: [1.0, 1.0],
                    }, // Bottom Right
                    Vertex {
                        position: [
                            (sx * cos_a - sy * sin_a) + px,
                            (sx * sin_a + sy * cos_a) + py,
                            pz,
                        ],
                        tex_coords: [1.0, 0.0],
                    }, // Top Right
                ];

                vertices
            })
            .collect();
        (vertices, entities)
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let (depth_texture, depth_view) = Self::create_depth_texture(device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        world: &World,
        viewport_size: Vec2,
    ) -> anyhow::Result<()> {
        let (verticies, entities) = self.build_sprite_verticies(world);

        let camera_matrix =
            if let Some(camera_entity) = world.query_with2::<Camera, Transform>().first() {
                let camera = world.get_component::<Camera>(*camera_entity).unwrap();
                let transform = world.get_component::<Transform>(*camera_entity).unwrap();

                let scale_x = viewport_size.x / camera.reference_resolution.x as f32 * camera.zoom;
                let scale_y = viewport_size.y / camera.reference_resolution.y as f32 * camera.zoom;
                let scale = scale_x.min(scale_y);

                let half_width = (viewport_size.x / scale) / 2.0;
                let half_height = (viewport_size.y / scale) / 2.0;

                let projection = glam::Mat4::orthographic_rh(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    -1000.0,
                    1000.0,
                );

                let (_, _, angle) = transform.rotation.to_euler(glam::EulerRot::XYZ);
                let view = glam::Mat4::from_rotation_z(-angle)
                    * glam::Mat4::from_translation(-transform.position);
                projection * view
            } else {
                glam::Mat4::IDENTITY
            };

        // A non-finite camera matrix (e.g. a script wrote a NaN/inf into the
        // camera's transform) would map every vertex off-screen and black out
        // the whole scene. Fall back to the last good matrix so a single bad
        // transform degrades gracefully instead of killing all rendering.
        let camera_matrix = if camera_matrix.is_finite() {
            self.last_camera_matrix = camera_matrix;
            camera_matrix
        } else {
            log::warn!("Camera matrix was non-finite; using last valid camera matrix");
            self.last_camera_matrix
        };

        let camera_color = if let Some(camera_entity) = world.query_with::<Camera>().first() {
            let camera = world.get_component::<Camera>(*camera_entity).unwrap();
            let color = camera.background_color;
            color
        } else {
            [0, 0, 0, 0]
        };

        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&camera_matrix.to_cols_array()),
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: camera_color[0] as f64 / 255.0,
                            g: camera_color[1] as f64 / 255.0,
                            b: camera_color[2] as f64 / 255.0,
                            a: camera_color[3] as f64 / 255.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            if !verticies.is_empty() {
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verticies),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

                for (index, entity) in entities.iter().enumerate() {
                    let sprite = world.get_component::<Sprite>(*entity).unwrap();
                    let start = (index * 6) as u32;
                    let end = start + 6;

                    if let Some((bind_group, _)) = self.gpu_textures.get(&sprite.asset_path) {
                        render_pass.set_bind_group(0, bind_group, &[]);
                        render_pass.draw(start..end, 0..1);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn upload_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        textures: &HashMap<String, Texture>,
    ) {
        for (path, texture) in textures {
            if !self.gpu_textures.contains_key(path) {
                let dimensions = texture.rgba_image.dimensions();

                let texture_size = wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    // All textures are stored as 3D, we represent our 2D texture
                    // by setting depth to 1.
                    depth_or_array_layers: 1,
                };

                let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    // Most images are stored using sRGB, so we need to reflect that here.
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                    // COPY_DST means that we want to copy data to this texture
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    label: Some("diffuse_texture"),
                    // This is the same as with the SurfaceConfig. It
                    // specifies what texture formats can be used to
                    // create TextureViews for this texture. The base
                    // texture format (Rgba8UnormSrgb in this case) is
                    // always supported. Note that using a different
                    // texture format is not supported on the WebGL2
                    // backend.
                    view_formats: &[],
                });

                queue.write_texture(
                    // Tells wgpu where to copy the pixel data
                    wgpu::TexelCopyTextureInfo {
                        texture: &diffuse_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    // The actual pixel data
                    &texture.rgba_image.as_raw(),
                    // The layout of the texture
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * dimensions.0),
                        rows_per_image: Some(dimensions.1),
                    },
                    texture_size,
                );

                let diffuse_texture_view =
                    diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                    ..Default::default()
                });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                });

                self.gpu_textures.insert(
                    path.clone(),
                    (
                        bind_group,
                        Vec2::new(dimensions.0 as f32, dimensions.1 as f32),
                    ),
                );
            }
        }
    }
}
