//! GPU-accelerated visual effects.

use super::shaders;
use crate::geometry::{Color, Point, Rect};
use wgpu;

/// GPU effects pipeline for blur, shadow, and glow effects.
pub struct EffectsPipeline {
    blur_pipeline: wgpu::RenderPipeline,
    blur_bind_group_layout: wgpu::BindGroupLayout,
    shadow_pipeline: wgpu::RenderPipeline,
    shadow_bind_group_layout: wgpu::BindGroupLayout,
    glow_pipeline: wgpu::RenderPipeline,
    glow_bind_group_layout: wgpu::BindGroupLayout,

    // Queued effects
    blur_queue: Vec<BlurEffect>,
    shadow_queue: Vec<ShadowEffect>,
    glow_queue: Vec<GlowEffect>,

    // Intermediate textures for multi-pass effects
    ping_pong_textures: Option<(wgpu::Texture, wgpu::Texture)>,
}

/// A blur effect to apply.
#[derive(Debug, Clone)]
pub struct BlurEffect {
    /// Region to blur.
    pub rect: Rect,
    /// Blur radius in pixels.
    pub radius: f32,
    /// Number of passes (more = smoother but slower).
    pub passes: u32,
}

/// A shadow effect to apply.
#[derive(Debug, Clone)]
pub struct ShadowEffect {
    /// Region casting the shadow.
    pub rect: Rect,
    /// Shadow color.
    pub color: Color,
    /// Blur radius.
    pub blur: f32,
    /// Shadow offset.
    pub offset: Point,
    /// Shadow spread.
    pub spread: f32,
}

/// A glow effect to apply.
#[derive(Debug, Clone)]
pub struct GlowEffect {
    /// Region to apply glow to.
    pub rect: Rect,
    /// Glow color.
    pub color: Color,
    /// Glow intensity.
    pub intensity: f32,
    /// Glow radius.
    pub radius: f32,
}

impl EffectsPipeline {
    /// Create a new effects pipeline.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // Create blur pipeline
        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::BLUR_H_SHADER.into()),
        });

        let blur_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blur Bind Group Layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let blur_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blur Pipeline Layout"),
            bind_group_layouts: &[&blur_bind_group_layout],
            push_constant_ranges: &[],
        });

        let blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blur Pipeline"),
            layout: Some(&blur_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blur_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blur_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create shadow pipeline
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::SHADOW_SHADER.into()),
        });

        let shadow_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shadow Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Pipeline Layout"),
            bind_group_layouts: &[&shadow_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[super::pipeline::RectVertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shadow_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create glow pipeline
        let glow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Glow Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::GLOW_SHADER.into()),
        });

        let glow_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glow Bind Group Layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let glow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Glow Pipeline Layout"),
            bind_group_layouts: &[&glow_bind_group_layout],
            push_constant_ranges: &[],
        });

        let glow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Glow Pipeline"),
            layout: Some(&glow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &glow_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &glow_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            blur_pipeline,
            blur_bind_group_layout,
            shadow_pipeline,
            shadow_bind_group_layout,
            glow_pipeline,
            glow_bind_group_layout,
            blur_queue: Vec::new(),
            shadow_queue: Vec::new(),
            glow_queue: Vec::new(),
            ping_pong_textures: None,
        }
    }

    /// Queue a blur effect.
    pub fn queue_blur(&mut self, rect: Rect, radius: f32) {
        self.blur_queue.push(BlurEffect {
            rect,
            radius,
            passes: 2,
        });
    }

    /// Queue a shadow effect.
    pub fn queue_shadow(&mut self, rect: Rect, color: Color, blur: f32, offset: Point) {
        self.shadow_queue.push(ShadowEffect {
            rect,
            color,
            blur,
            offset,
            spread: 0.0,
        });
    }

    /// Queue a glow effect.
    pub fn queue_glow(&mut self, rect: Rect, color: Color, intensity: f32, radius: f32) {
        self.glow_queue.push(GlowEffect {
            rect,
            color,
            intensity,
            radius,
        });
    }

    /// Clear all queued effects.
    pub fn clear(&mut self) {
        self.blur_queue.clear();
        self.shadow_queue.clear();
        self.glow_queue.clear();
    }

    /// Get queued blur effects.
    pub fn blur_effects(&self) -> &[BlurEffect] {
        &self.blur_queue
    }

    /// Get queued shadow effects.
    pub fn shadow_effects(&self) -> &[ShadowEffect] {
        &self.shadow_queue
    }

    /// Get queued glow effects.
    pub fn glow_effects(&self) -> &[GlowEffect] {
        &self.glow_queue
    }

    /// Get the blur pipeline.
    pub fn blur_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.blur_pipeline
    }

    /// Get the shadow pipeline.
    pub fn shadow_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.shadow_pipeline
    }

    /// Get the glow pipeline.
    pub fn glow_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.glow_pipeline
    }

    /// Get bind group layouts.
    pub fn blur_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.blur_bind_group_layout
    }

    pub fn shadow_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.shadow_bind_group_layout
    }

    pub fn glow_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.glow_bind_group_layout
    }
}
