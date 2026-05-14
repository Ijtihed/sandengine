use crate::sim::grid3d::Grid3D;
use crate::sim::material::Cell;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CubeVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct VoxelInstance {
    pub offset: [f32; 3],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    vp: [[f32; 4]; 4],
    light_dir: [f32; 3],
    _pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct LineVertex {
    position: [f32; 3],
    color: [f32; 4],
}

pub struct Renderer3D {
    cube_vbuf: wgpu::Buffer,
    cube_vertex_count: u32,
    instance_buf: wgpu::Buffer,
    instance_capacity: u32,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    grid_vbuf: wgpu::Buffer,
    grid_vertex_count: u32,
    grid_pipeline: wgpu::RenderPipeline,
}

impl Renderer3D {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, grid_size: usize) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("voxel_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/voxel.wgsl").into()),
        });

        let cube_vertices = Self::unit_cube_vertices();
        let cube_vertex_count = cube_vertices.len() as u32;
        let cube_vbuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_vbuf"),
            size: (cube_vertices.len() * std::mem::size_of::<CubeVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&cube_vbuf, 0, bytemuck::cast_slice(&cube_vertices));

        let max_instances: u32 = 1_000_000;
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance_buf"),
            size: (max_instances as usize * std::mem::size_of::<VoxelInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buf"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("voxel_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("voxel_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("voxel_pl"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let cube_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CubeVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
            ],
        };

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VoxelInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 12,
                    shader_location: 3,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("voxel_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[cube_vertex_layout, instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Floor grid
        let grid_verts = Self::floor_grid_vertices(grid_size);
        let grid_vertex_count = grid_verts.len() as u32;
        let grid_vbuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grid_line_vbuf"),
            size: (grid_verts.len() * std::mem::size_of::<LineVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&grid_vbuf, 0, bytemuck::cast_slice(&grid_verts));

        let grid_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 12,
                    shader_location: 1,
                },
            ],
        };

        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid_line_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("grid_vs"),
                buffers: &[grid_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("grid_fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            cube_vbuf,
            cube_vertex_count,
            instance_buf,
            instance_capacity: max_instances,
            uniform_buf,
            bind_group,
            pipeline,
            grid_vbuf,
            grid_vertex_count,
            grid_pipeline,
        }
    }

    pub fn build_instances(&self, grid: &Grid3D) -> Vec<VoxelInstance> {
        let mut instances = Vec::new();
        for y in 0..grid.sy {
            for z in 0..grid.sz {
                for x in 0..grid.sx {
                    let cell = grid.get(x, y, z);
                    if cell.kind == Cell::Air {
                        continue;
                    }
                    if !grid.is_surface(x, y, z) {
                        continue;
                    }
                    instances.push(VoxelInstance {
                        offset: [x as f32, y as f32, z as f32],
                        color: cell.to_rgba_f32(),
                    });
                }
            }
        }
        instances
    }

    pub fn prepare(&self, queue: &wgpu::Queue, vp: Mat4, instances: &[VoxelInstance]) {
        let uniforms = Uniforms {
            vp: vp.to_cols_array_2d(),
            light_dir: Vec3::new(0.5, 0.8, 0.3).normalize().to_array(),
            _pad: 0.0,
        };
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));

        let count = (instances.len() as u32).min(self.instance_capacity);
        if count > 0 {
            queue.write_buffer(
                &self.instance_buf,
                0,
                bytemuck::cast_slice(&instances[..count as usize]),
            );
        }
    }

    pub fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>, instance_count: u32) {
        // Floor grid
        rpass.set_pipeline(&self.grid_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.grid_vbuf.slice(..));
        rpass.draw(0..self.grid_vertex_count, 0..1);

        // Voxels
        let count = instance_count.min(self.instance_capacity);
        if count > 0 {
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.cube_vbuf.slice(..));
            rpass.set_vertex_buffer(1, self.instance_buf.slice(..));
            rpass.draw(0..self.cube_vertex_count, 0..count);
        }
    }

    fn unit_cube_vertices() -> Vec<CubeVertex> {
        let s = 0.5;
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            ([1.0, 0.0, 0.0], [[s, -s, -s], [s, s, -s], [s, s, s], [s, -s, s]]),
            ([-1.0, 0.0, 0.0], [[-s, -s, s], [-s, s, s], [-s, s, -s], [-s, -s, -s]]),
            ([0.0, 1.0, 0.0], [[-s, s, -s], [-s, s, s], [s, s, s], [s, s, -s]]),
            ([0.0, -1.0, 0.0], [[-s, -s, s], [-s, -s, -s], [s, -s, -s], [s, -s, s]]),
            ([0.0, 0.0, 1.0], [[-s, -s, s], [s, -s, s], [s, s, s], [-s, s, s]]),
            ([0.0, 0.0, -1.0], [[s, -s, -s], [-s, -s, -s], [-s, s, -s], [s, s, -s]]),
        ];

        let mut verts = Vec::with_capacity(36);
        for (normal, corners) in &faces {
            for &i in &[0, 1, 2, 0, 2, 3] {
                verts.push(CubeVertex {
                    position: corners[i],
                    normal: *normal,
                });
            }
        }
        verts
    }

    fn floor_grid_vertices(size: usize) -> Vec<LineVertex> {
        let mut verts = Vec::new();
        let s = size as f32;
        let step = if size > 64 { 8 } else { 4 };
        for i in (0..=size).step_by(step) {
            let f = i as f32;
            let a = if i % (step * 4) == 0 { 0.5 } else { 0.25 };
            let color = [0.4, 0.4, 0.4, a];
            verts.push(LineVertex { position: [0.0, 0.0, f], color });
            verts.push(LineVertex { position: [s, 0.0, f], color });
            verts.push(LineVertex { position: [f, 0.0, 0.0], color });
            verts.push(LineVertex { position: [f, 0.0, s], color });
        }
        verts
    }
}
