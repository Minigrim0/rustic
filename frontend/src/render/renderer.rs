use wgpu_glyph::{Section, Text, ab_glyph};
use winit::{dpi::PhysicalSize, window::Window};

use crate::scenes::Scene;

const FONT_BYTES: &[u8] = include_bytes!("../../res/fonts/PressStart2P-Regular.ttf");

pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
}

impl<'a> Renderer<'a> {
    pub fn width(&self) -> f32 {
        self.config.width as f32
    }

    #[allow(dead_code)]
    pub fn height(&self) -> f32 {
        self.config.height as f32
    }

    pub async fn new(window: &'a Window, size: PhysicalSize<u32>) -> Renderer<'a> {
        log::warn!("size: {:?}", size);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            config.format,
            &[super::vertex::Vertex::DESC],
            wgpu::include_wgsl!("../../res/shaders/textured.vert.wgsl"),
            wgpu::include_wgsl!("../../res/shaders/textured.frag.wgsl"),
        );

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: super::vertex::Vertex::SIZE * 4 * 3,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<u32>() as wgpu::BufferAddress * 6 * 3,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let font = ab_glyph::FontArc::try_from_slice(FONT_BYTES).unwrap();
        let glyph_brush =
            wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&device, config.format);
        let staging_belt = wgpu::util::StagingBelt::new(1024);

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            glyph_brush,
            staging_belt,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, scene: &Scene) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let num_indices = {
            let (stg_vertex, stg_index, num_indices) = scene.get_vertices().build(&self.device);

            stg_vertex.copy_to_buffer(&mut encoder, &self.vertex_buffer);
            stg_index.copy_to_buffer(&mut encoder, &self.index_buffer);
            num_indices
        };

        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&Default::default());
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                if num_indices != 0 {
                    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.set_pipeline(&self.pipeline);
                    render_pass.draw_indexed(0..num_indices, 0, 0..1);
                }

                std::mem::drop(render_pass);

                draw_text(
                    &crate::widgets::prelude::Text {
                        position: cgmath::Vector2::new(200.0, 100.0),
                        bounds: cgmath::Vector2::new(100.0, 100.0),
                        color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
                        text: "Hello, World!".to_string(),
                        size: 12.0,
                        visible: true,
                        focused: false,
                        centered: true,
                    },
                    &mut self.glyph_brush,
                );

                self.glyph_brush
                    .draw_queued(
                        &self.device,
                        &mut self.staging_belt,
                        &mut encoder,
                        &view,
                        self.config.width,
                        self.config.height,
                    )
                    .unwrap();

                self.staging_belt.finish();
                // submit will accept anything that implements IntoIter
                self.queue.submit(std::iter::once(encoder.finish()));
                frame.present();
            }
            Err(wgpu::SurfaceError::Outdated) => {
                log::info!("Outdated surface texture");
                self.surface.configure(&self.device, &self.config);
            }
            Err(e) => {
                log::error!("Error: {}", e);
            }
        }
    }
}

fn draw_text(text: &crate::widgets::prelude::Text, glyph_brush: &mut wgpu_glyph::GlyphBrush<()>) {
    let layout = wgpu_glyph::Layout::default().h_align(if text.centered {
        wgpu_glyph::HorizontalAlign::Center
    } else {
        wgpu_glyph::HorizontalAlign::Left
    });

    let section =
        Section {
            screen_position: text.position.into(),
            bounds: text.bounds.into(),
            layout,
            ..Section::default()
        }
        .add_text(Text::new(&text.text).with_color(text.color).with_scale(
            if text.focused {
                text.size + 8.0
            } else {
                text.size
            },
        ));

    glyph_brush.queue(section);
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    vs_src: wgpu::ShaderModuleDescriptor,
    fs_src: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let vs_module = device.create_shader_module(vs_src);
    let fs_module = device.create_shader_module(fs_src);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: Some("main"),
            buffers: &vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
        cache: None,
    })
}
