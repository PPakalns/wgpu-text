use std::num::NonZeroU32;

use glyph_brush::Rectangle;
use wgpu::util::DeviceExt;

pub struct Uniform {
    matrix_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Uniform {
    pub fn new(device: &wgpu::Device, tex_width: u32, tex_height: u32, window_size: (f32, f32)) -> Self {
        let texture = Self::new_cache_texture(device, tex_width, tex_height);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("wgpu-text Cache Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("wgpu-text Matrix Uniform Buffer"),
            contents: bytemuck::cast_slice(&ortho(window_size.0, window_size.1)),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wgpu-text Matrix, Texture, Sampler Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wgpu-rs Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            matrix_buffer,
            texture,
            sampler,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn recreate_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture = Self::new_cache_texture(device, width, height);
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wgpu-rs Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
    }

    pub fn update_matrix(&mut self, width: f32, height: f32, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.matrix_buffer,
            0,
            bytemuck::cast_slice(&ortho(width, height)),
        );
    }

    pub fn update_texture(&mut self, size: Rectangle<u32>, data: &[u8], queue: &wgpu::Queue) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: size.min[0],
                    y: size.min[1],
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(size.width()),
                rows_per_image: NonZeroU32::new(size.height()),
            },
            wgpu::Extent3d {
                width: size.width(),
                height: size.height(),
                depth_or_array_layers: 1,
            },
        )
    }

    fn new_cache_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("wgpu-text Cache Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        })
    }
}

#[rustfmt::skip]
fn ortho(width: f32, height: f32) -> [f32; 16] {
    [
        2.0 / width, 0.0,          0.0, 0.0,
        0.0,        -2.0 / height, 0.0, 0.0,
        0.0,         0.0,          1.0, 0.0,
       -1.0,         1.0,          0.0, 1.0,
    ]
}
