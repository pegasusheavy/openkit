//! Rendering pipeline benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use openkit::geometry::{Rect, Color, BorderRadius, Point};
use openkit::render::{Painter, DrawCommand};

fn bench_painter_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("painter_operations");
    
    // Fill rect
    group.bench_function("fill_rect", |b| {
        let mut painter = Painter::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        let color = Color::from_rgb8(59, 130, 246);
        
        b.iter(|| {
            painter.fill_rect(black_box(rect), black_box(color));
        });
    });
    
    // Fill rounded rect
    group.bench_function("fill_rounded_rect", |b| {
        let mut painter = Painter::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        let color = Color::from_rgb8(59, 130, 246);
        let radius = BorderRadius::uniform(8.0);
        
        b.iter(|| {
            painter.fill_rounded_rect(black_box(rect), black_box(color), black_box(radius));
        });
    });
    
    // Draw text
    group.bench_function("draw_text", |b| {
        let mut painter = Painter::new();
        let position = Point::new(10.0, 30.0);
        let color = Color::WHITE;
        
        b.iter(|| {
            painter.draw_text(black_box("Hello, World!"), black_box(position), black_box(color), black_box(16.0));
        });
    });
    
    // Draw line
    group.bench_function("draw_line", |b| {
        let mut painter = Painter::new();
        let from = Point::new(0.0, 0.0);
        let to = Point::new(100.0, 100.0);
        let color = Color::WHITE;
        
        b.iter(|| {
            painter.draw_line(black_box(from), black_box(to), black_box(color), black_box(2.0));
        });
    });
    
    group.finish();
}

fn bench_command_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_batching");
    
    for count in [10, 100, 500, 1000] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("rects", count), &count, |b, &count| {
            b.iter(|| {
                let mut painter = Painter::new();
                let color = Color::from_rgb8(59, 130, 246);
                
                for i in 0..count {
                    let rect = Rect::new(
                        (i % 10) as f32 * 110.0,
                        (i / 10) as f32 * 60.0,
                        100.0,
                        50.0,
                    );
                    painter.fill_rect(rect, color);
                }
                
                black_box(painter.finish())
            })
        });
    }
    
    group.finish();
}

fn bench_transform_stack(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_stack");
    
    group.bench_function("save_restore", |b| {
        let mut painter = Painter::new();
        
        b.iter(|| {
            painter.save();
            painter.translate(10.0, 10.0);
            painter.scale(1.5, 1.5);
            painter.restore();
        });
    });
    
    group.bench_function("nested_transforms", |b| {
        let mut painter = Painter::new();
        
        b.iter(|| {
            for _ in 0..5 {
                painter.save();
                painter.translate(black_box(10.0), black_box(10.0));
            }
            for _ in 0..5 {
                painter.restore();
            }
        });
    });
    
    group.finish();
}

fn bench_clip_stack(c: &mut Criterion) {
    let mut group = c.benchmark_group("clip_stack");
    
    group.bench_function("push_pop_clip", |b| {
        let mut painter = Painter::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        
        b.iter(|| {
            painter.push_clip(black_box(rect));
            painter.pop_clip();
        });
    });
    
    group.finish();
}

fn bench_draw_command_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_command_creation");
    
    group.bench_function("rect_command", |b| {
        b.iter(|| {
            black_box(DrawCommand::Rect {
                rect: Rect::new(0.0, 0.0, 100.0, 50.0),
                color: Color::from_rgb8(59, 130, 246),
                radius: BorderRadius::ZERO,
            })
        })
    });
    
    group.bench_function("text_command", |b| {
        b.iter(|| {
            black_box(DrawCommand::Text {
                text: "Hello, World!".to_string(),
                position: Point::new(10.0, 30.0),
                color: Color::WHITE,
                size: 16.0,
            })
        })
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    bench_painter_operations,
    bench_command_batching,
    bench_transform_stack,
    bench_clip_stack,
    bench_draw_command_creation
);
criterion_main!(benches);
