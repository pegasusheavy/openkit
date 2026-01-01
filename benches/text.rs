//! Text rendering benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use openkit::render::TextRenderer;

const SHORT_TEXT: &str = "Hello";
const MEDIUM_TEXT: &str = "The quick brown fox jumps over the lazy dog.";
const LONG_TEXT: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.";

fn bench_text_measurement(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_measurement");
    let renderer = TextRenderer::new();
    
    // Short text
    group.throughput(Throughput::Bytes(SHORT_TEXT.len() as u64));
    group.bench_with_input(BenchmarkId::new("short", SHORT_TEXT.len()), SHORT_TEXT, |b, text| {
        b.iter(|| renderer.measure(black_box(text), black_box(16.0)))
    });
    
    // Medium text
    group.throughput(Throughput::Bytes(MEDIUM_TEXT.len() as u64));
    group.bench_with_input(BenchmarkId::new("medium", MEDIUM_TEXT.len()), MEDIUM_TEXT, |b, text| {
        b.iter(|| renderer.measure(black_box(text), black_box(16.0)))
    });
    
    // Long text
    group.throughput(Throughput::Bytes(LONG_TEXT.len() as u64));
    group.bench_with_input(BenchmarkId::new("long", LONG_TEXT.len()), LONG_TEXT, |b, text| {
        b.iter(|| renderer.measure(black_box(text), black_box(16.0)))
    });
    
    group.finish();
}

fn bench_font_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_sizes");
    let renderer = TextRenderer::new();
    
    for size in [12.0, 14.0, 16.0, 18.0, 24.0, 32.0, 48.0] {
        group.bench_with_input(BenchmarkId::new("measure", size as u32), &size, |b, &size| {
            b.iter(|| renderer.measure(black_box(MEDIUM_TEXT), black_box(size)))
        });
    }
    
    group.finish();
}

fn bench_text_rasterization(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_rasterization");
    let mut renderer = TextRenderer::new();
    let color = [255u8, 255, 255, 255];
    
    // Short text
    group.bench_function("short", |b| {
        b.iter(|| renderer.rasterize(black_box(SHORT_TEXT), black_box(16.0), black_box(color)))
    });
    
    // Medium text
    group.bench_function("medium", |b| {
        b.iter(|| renderer.rasterize(black_box(MEDIUM_TEXT), black_box(16.0), black_box(color)))
    });
    
    group.finish();
}

fn bench_unicode_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("unicode");
    let renderer = TextRenderer::new();
    
    // ASCII only
    group.bench_function("ascii", |b| {
        b.iter(|| renderer.measure(black_box("Hello World"), black_box(16.0)))
    });
    
    // With emojis
    group.bench_function("emojis", |b| {
        b.iter(|| renderer.measure(black_box("Hello 🌍 World 🚀"), black_box(16.0)))
    });
    
    // CJK characters
    group.bench_function("cjk", |b| {
        b.iter(|| renderer.measure(black_box("你好世界"), black_box(16.0)))
    });
    
    // Mixed script
    group.bench_function("mixed", |b| {
        b.iter(|| renderer.measure(black_box("Hello 你好 مرحبا"), black_box(16.0)))
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    bench_text_measurement,
    bench_font_sizes,
    bench_text_rasterization,
    bench_unicode_handling
);
criterion_main!(benches);
