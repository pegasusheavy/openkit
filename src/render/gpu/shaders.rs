//! WGSL shaders for GPU-accelerated rendering.

/// Shader for rendering rectangles with rounded corners and gradients.
pub const RECT_SHADER: &str = r#"
// Vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) rect_bounds: vec4<f32>,  // x, y, width, height
    @location(4) corner_radii: vec4<f32>, // top_left, top_right, bottom_right, bottom_left
    @location(5) params: vec4<f32>,       // gradient_angle, border_width, flags, unused
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) rect_bounds: vec4<f32>,
    @location(3) corner_radii: vec4<f32>,
    @location(4) local_pos: vec2<f32>,
    @location(5) params: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    out.rect_bounds = in.rect_bounds;
    out.corner_radii = in.corner_radii;
    out.local_pos = in.uv * in.rect_bounds.zw;
    out.params = in.params;
    return out;
}

// Fragment shader
fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    // Select corner radius based on quadrant
    var r: f32;
    if (p.x > 0.0) {
        if (p.y > 0.0) {
            r = radii.z; // bottom-right
        } else {
            r = radii.y; // top-right
        }
    } else {
        if (p.y > 0.0) {
            r = radii.w; // bottom-left
        } else {
            r = radii.x; // top-left
        }
    }

    let q = abs(p) - size + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = in.rect_bounds.zw;
    let center = size * 0.5;
    let p = in.local_pos - center;

    // Calculate SDF for rounded rectangle
    let d = rounded_box_sdf(p, center, in.corner_radii);

    // Anti-aliased edge
    let aa = 1.0;
    let alpha = 1.0 - smoothstep(-aa, aa, d);

    // Border rendering
    let border_width = in.params.y;
    if (border_width > 0.0) {
        let inner_d = rounded_box_sdf(p, center - vec2<f32>(border_width), in.corner_radii - vec4<f32>(border_width));
        let border_alpha = smoothstep(-aa, aa, inner_d);
        return vec4<f32>(in.color.rgb, in.color.a * alpha * border_alpha);
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
"#;

/// Shader for rendering gradients.
pub const GRADIENT_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color_start: vec4<f32>,
    @location(3) color_end: vec4<f32>,
    @location(4) rect_bounds: vec4<f32>,
    @location(5) corner_radii: vec4<f32>,
    @location(6) gradient_params: vec4<f32>, // angle, type, unused, unused
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color_start: vec4<f32>,
    @location(2) color_end: vec4<f32>,
    @location(3) rect_bounds: vec4<f32>,
    @location(4) corner_radii: vec4<f32>,
    @location(5) local_pos: vec2<f32>,
    @location(6) gradient_params: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color_start = in.color_start;
    out.color_end = in.color_end;
    out.rect_bounds = in.rect_bounds;
    out.corner_radii = in.corner_radii;
    out.local_pos = in.uv * in.rect_bounds.zw;
    out.gradient_params = in.gradient_params;
    return out;
}

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    var r: f32;
    if (p.x > 0.0) {
        if (p.y > 0.0) { r = radii.z; } else { r = radii.y; }
    } else {
        if (p.y > 0.0) { r = radii.w; } else { r = radii.x; }
    }
    let q = abs(p) - size + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = in.rect_bounds.zw;
    let center = size * 0.5;
    let p = in.local_pos - center;

    // Calculate gradient factor based on angle
    let angle = in.gradient_params.x;
    let dir = vec2<f32>(cos(angle), sin(angle));
    let gradient_t = dot(in.uv - 0.5, dir) + 0.5;

    // Interpolate colors
    let color = mix(in.color_start, in.color_end, clamp(gradient_t, 0.0, 1.0));

    // Apply rounded rect mask
    let d = rounded_box_sdf(p, center, in.corner_radii);
    let alpha = 1.0 - smoothstep(-1.0, 1.0, d);

    return vec4<f32>(color.rgb, color.a * alpha);
}
"#;

/// Shader for rendering text (glyph atlas).
pub const TEXT_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0) var t_glyph: texture_2d<f32>;
@group(0) @binding(1) var s_glyph: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(t_glyph, s_glyph, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
"#;

/// Shader for rendering images/textures.
pub const IMAGE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tint: vec4<f32>,
    @location(3) rect_bounds: vec4<f32>,
    @location(4) corner_radii: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
    @location(2) rect_bounds: vec4<f32>,
    @location(3) corner_radii: vec4<f32>,
    @location(4) local_pos: vec2<f32>,
}

@group(0) @binding(0) var t_image: texture_2d<f32>;
@group(0) @binding(1) var s_image: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.tint = in.tint;
    out.rect_bounds = in.rect_bounds;
    out.corner_radii = in.corner_radii;
    out.local_pos = in.uv * in.rect_bounds.zw;
    return out;
}

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    var r: f32;
    if (p.x > 0.0) {
        if (p.y > 0.0) { r = radii.z; } else { r = radii.y; }
    } else {
        if (p.y > 0.0) { r = radii.w; } else { r = radii.x; }
    }
    let q = abs(p) - size + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = in.rect_bounds.zw;
    let center = size * 0.5;
    let p = in.local_pos - center;

    let tex_color = textureSample(t_image, s_image, in.uv);
    let color = tex_color * in.tint;

    // Apply rounded rect mask
    let d = rounded_box_sdf(p, center, in.corner_radii);
    let alpha = 1.0 - smoothstep(-1.0, 1.0, d);

    return vec4<f32>(color.rgb, color.a * alpha);
}
"#;

/// Gaussian blur shader (horizontal pass).
pub const BLUR_H_SHADER: &str = r#"
struct BlurParams {
    direction: vec2<f32>,
    radius: f32,
    _padding: f32,
}

@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_input: sampler;
@group(0) @binding(2) var<uniform> params: BlurParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Full-screen triangle
    let x = f32((vertex_index & 1u) << 2u) - 1.0;
    let y = f32((vertex_index & 2u) << 1u) - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(t_input));
    let pixel_size = 1.0 / tex_size;

    var color = vec4<f32>(0.0);
    var total_weight = 0.0;

    let radius = i32(params.radius);
    for (var i = -radius; i <= radius; i++) {
        let offset = vec2<f32>(f32(i)) * params.direction * pixel_size;
        let weight = exp(-f32(i * i) / (2.0 * params.radius * params.radius));
        color += textureSample(t_input, s_input, in.uv + offset) * weight;
        total_weight += weight;
    }

    return color / total_weight;
}
"#;

/// Shadow shader.
pub const SHADOW_SHADER: &str = r#"
struct ShadowParams {
    color: vec4<f32>,
    offset: vec2<f32>,
    blur: f32,
    spread: f32,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) rect_bounds: vec4<f32>,
    @location(3) corner_radii: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) rect_bounds: vec4<f32>,
    @location(2) corner_radii: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: ShadowParams;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position + params.offset / 100.0, 0.0, 1.0);
    out.local_pos = in.uv * in.rect_bounds.zw;
    out.rect_bounds = in.rect_bounds;
    out.corner_radii = in.corner_radii;
    return out;
}

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    var r: f32;
    if (p.x > 0.0) {
        if (p.y > 0.0) { r = radii.z; } else { r = radii.y; }
    } else {
        if (p.y > 0.0) { r = radii.w; } else { r = radii.x; }
    }
    let q = abs(p) - size + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = in.rect_bounds.zw + vec2<f32>(params.spread * 2.0);
    let center = size * 0.5;
    let p = in.local_pos - center + vec2<f32>(params.spread);

    let d = rounded_box_sdf(p, center, in.corner_radii);

    // Soft shadow falloff
    let shadow_alpha = 1.0 - smoothstep(-params.blur, params.blur, d);

    return vec4<f32>(params.color.rgb, params.color.a * shadow_alpha);
}
"#;

/// Glow effect shader.
pub const GLOW_SHADER: &str = r#"
struct GlowParams {
    color: vec4<f32>,
    intensity: f32,
    radius: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_input: sampler;
@group(0) @binding(2) var<uniform> params: GlowParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 2u) - 1.0;
    let y = f32((vertex_index & 2u) << 1u) - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(t_input));
    let pixel_size = 1.0 / tex_size;

    var glow = vec4<f32>(0.0);
    var total_weight = 0.0;

    let radius = i32(params.radius);
    for (var y = -radius; y <= radius; y++) {
        for (var x = -radius; x <= radius; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * pixel_size;
            let dist = length(vec2<f32>(f32(x), f32(y)));
            let weight = exp(-dist * dist / (2.0 * params.radius * params.radius));
            glow += textureSample(t_input, s_input, in.uv + offset) * weight;
            total_weight += weight;
        }
    }

    glow = glow / total_weight;
    let original = textureSample(t_input, s_input, in.uv);

    // Add glow
    return original + glow * params.color * params.intensity;
}
"#;
