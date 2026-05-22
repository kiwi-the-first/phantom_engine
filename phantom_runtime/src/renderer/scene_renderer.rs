use std::collections::HashMap;

use glam::Vec2;
use phantom_core::ecs::World;
use phantom_core::ecs::components::camera::{self, Camera};
use phantom_core::ecs::components::{Sprite, Transform};
use wgpu::util::DeviceExt;

use crate::asset_manager::asset_types::texture::Texture;
use crate::renderer::vertex::Vertex;

pub struct SceneRenderer {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    gpu_textures: HashMap<String, wgpu::BindGroup>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
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
            depth_stencil: None, // 1.
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
        }
    }

    pub fn build_sprite_verticies(&self, world: &World) -> Vec<Vertex> {
        let entities = world.query_with2::<Sprite, Transform>();
        let vertices = entities
            .iter()
            .flat_map(|entity| {
                let transform = world.get_component::<Transform>(*entity).unwrap();
                let (_, _, angle) = transform.rotation.to_euler(glam::EulerRot::XYZ);
                let cos_a = angle.cos();
                let sin_a = angle.sin();

                let sx = transform.scale.x;
                let sy = transform.scale.y;
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
        vertices
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
        let verticies = self.build_sprite_verticies(world);

        let camera_matrix =
            if let Some(camera_entity) = world.query_with2::<Camera, Transform>().first() {
                let camera = world.get_component::<Camera>(*camera_entity).unwrap();
                let transform = world.get_component::<Transform>(*camera_entity).unwrap();
                let left = transform.position.x - (viewport_size.x / 2.0) / camera.zoom;
                let right = transform.position.x + (viewport_size.x / 2.0) / camera.zoom;
                let bottom = transform.position.y - (viewport_size.y / 2.0) / camera.zoom;
                let top = transform.position.y + (viewport_size.y / 2.0) / camera.zoom;
                let near = -1000.0;
                let far = 1000.0;
                glam::Mat4::orthographic_rh(left, right, bottom, top, near, far)
            } else {
                glam::Mat4::IDENTITY
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
                depth_stencil_attachment: None,
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
                let entities = world.query_with2::<Sprite, Transform>();
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

                for (index, entity) in entities.iter().enumerate() {
                    let sprite = world.get_component::<Sprite>(*entity).unwrap();
                    let start = (index * 6) as u32;
                    let end = start + 6;

                    if let Some(bind_group) = self.gpu_textures.get(&sprite.asset_path) {
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

                self.gpu_textures.insert(path.clone(), bind_group);
            }
        }
    }
}
