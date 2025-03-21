use std::{borrow::Cow, fs, io::read_to_string};

use pixels::{
    check_texture_size,
    wgpu::{self, util::DeviceExt},
    Pixels, TextureError,
};

pub struct Renderer {
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    time_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    cutout_buffer: wgpu::Buffer,
    // stores the current dimensions of the window, to create the
    // constants needed to transform the pixels to size properly.
    width: u32,
    height: u32,
    time: f32,
}

#[derive(Copy, Clone)]
pub enum Shader {
    BASE,
    CRT,
    GB,
    HUE,
    THREED,
    INVERT,
}

impl Shader {
    // maps the enum to the location of the shader file.
    pub fn file(shader: Shader) -> &'static str {
        match shader {
            Shader::BASE => include_str!("shaders/base.wgsl"),
            Shader::CRT => include_str!("shaders/crt.wgsl"),
            Shader::GB => include_str!("shaders/gb.wgsl"),
            Shader::HUE => include_str!("shaders/hue.wgsl"),
            Shader::THREED => include_str!("shaders/3d.wgsl"),
            Shader::INVERT => include_str!("shaders/invert.wgsl"),
        }
    }

    // gives the name of the shader.
    pub fn name(shader: Shader) -> String {
        match shader {
            Shader::BASE => "base",
            Shader::CRT => "crt",
            Shader::GB => "gb",
            Shader::HUE => "hue",
            Shader::THREED => "3d",
            Shader::INVERT => "invert",
        }
        .to_owned()
    }
}

pub const SHADER_LIST: [Shader; 6] = [
    Shader::BASE,
    Shader::CRT,
    Shader::GB,
    Shader::HUE,
    Shader::THREED,
    Shader::INVERT,
];

impl Renderer {
    pub fn new(
        pixels: &Pixels,
        width: u32,
        height: u32,
        shader: Shader,
        b_width: u32,
        b_height: u32,
    ) -> Result<Self, TextureError> {
        let device = pixels.device();

        // shader module is loaded from the file declared by the enum above.
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Shader::file(shader).into()),
        });

        // Create a texture view that will be used as input
        // This will be used as the render target for the default scaling renderer
        let texture_view = create_texture_view(pixels, width, height)?;

        // Create a texture sampler using  nearest neighbor.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Renderer sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        // Create vertex buffer; array-of-array of position and texture coordinates
        let vertex_data: [[f32; 2]; 3] = [
            // One full-screen triangle used instead of two triangles that form a quad, as it leads to less overhead while rendering.
            [-1.0, -1.0], // Bottom-left
            [3.0, -1.0],  // Bottom-right
            [-1.0, 3.0],  // Top-left
        ];
        let vertex_data_slice = bytemuck::cast_slice(&vertex_data);

        // Allocates the vertex buffer, by storing the clip-space vertex poisitions and texture coordinates.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Renderer vertex buffer"),
            contents: vertex_data_slice,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: (vertex_data_slice.len() / vertex_data.len()) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        // Create uniform buffer
        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Renderer u_Time"),
            contents: &0.0_f32.to_ne_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let cutout_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cutout Region Buffer"),
            contents: bytemuck::cast_slice(&[
                0.0_f32,
                0.0_f32,
                1.0_f32,
                1.0_f32,
                b_width as f32,
                b_height as f32,
            ]), // Default: full screen
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout, which defines how textures / buffers will be accessed by the GPU via bindings.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<f32>() as u64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<[f32; 6]>() as u64
                        ),
                    },
                    count: None,
                },
            ],
        });

        // ties the texture, sampler and uniform buffers into a bind group.
        let bind_group = create_bind_group(
            device,
            &bind_group_layout,
            &texture_view,
            &sampler,
            &time_buffer,
            &cutout_buffer,
        );

        // Defines the layout of the pipe line.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Renderer pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Creates the Pipeline, as well as defining the entry points for both the fragment and vertex shader.
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Renderer pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: pixels.render_texture_format(),
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Ok(Self {
            texture_view,
            sampler,
            bind_group_layout,
            bind_group,
            render_pipeline,
            time_buffer,
            vertex_buffer,
            cutout_buffer,
            width,
            height,
            time: 0.,
        })
    }

    // returns the texture_view.
    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    /* reallocates textures and updates the bind group when the window is resized.
    needed so that the underlying default scaling renderer can pass in the new scaled up texture.*/

    pub fn resize(
        &mut self,
        pixels: &pixels::Pixels,
        width: u32,
        height: u32,
    ) -> Result<(), TextureError> {
        self.texture_view = create_texture_view(pixels, width, height)?;
        self.bind_group = create_bind_group(
            pixels.device(),
            &self.bind_group_layout,
            &self.texture_view,
            &self.sampler,
            &self.time_buffer,
            &self.cutout_buffer,
        );

        self.width = width;
        self.height = height;

        Ok(())
    }

    /* updates uniform buffer values regarding size of window and the size of screen,
    for transformations in the shader to prevent distortions, and the time buffer.*/

    pub fn update(&mut self, queue: &wgpu::Queue, clip_rect: (u32, u32, u32, u32)) {
        self.time += 0.01;
        queue.write_buffer(&self.time_buffer, 0, &self.time.to_ne_bytes());

        queue.write_buffer(
            &self.cutout_buffer,
            0,
            bytemuck::cast_slice(&[
                clip_rect.0 as f32 / self.width as f32,
                clip_rect.1 as f32 / self.height as f32,
                clip_rect.2 as f32 / self.width as f32,
                clip_rect.3 as f32 / self.height as f32,
            ]),
        );
    }

    /* actual render function. */
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
    ) {
        // render pass is started, setting the pipeline, bind group and drawing the vertices.
        // clears the frame buffer to black, and prepares the frame buffer to receive rendered pixels.
        // output is given to the render target.
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Renderer render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw(0..3, 0..1);
    }

    pub fn reset(&mut self) {
        self.time = 0.0
    }
}

// creates the new texture view and renders..
fn create_texture_view(
    pixels: &pixels::Pixels,
    width: u32,
    height: u32,
) -> Result<wgpu::TextureView, TextureError> {
    let device = pixels.device();
    check_texture_size(device, width, height)?;
    let texture_descriptor = wgpu::TextureDescriptor {
        label: None,
        size: pixels::wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: pixels.render_texture_format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    Ok(device
        .create_texture(&texture_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default()))
}

// creates the bind group.
fn create_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    time_buffer: &wgpu::Buffer,
    cutout_buffer: &wgpu::Buffer,
) -> pixels::wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: time_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: cutout_buffer.as_entire_binding(),
            },
        ],
    })
}
