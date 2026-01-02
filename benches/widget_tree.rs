//! Widget tree benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use openkit::geometry::{Rect, Size, Point, Color};
use openkit::widget::next_widget_id;
use openkit::css::ClassList;

/// Simulate widget ID generation
fn bench_widget_id_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("widget_id");
    
    group.bench_function("single", |b| {
        b.iter(|| black_box(next_widget_id()))
    });
    
    group.bench_function("batch_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(next_widget_id());
            }
        })
    });
    
    group.finish();
}

/// Simulate class list operations
fn bench_class_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("class_list");
    
    group.bench_function("create_empty", |b| {
        b.iter(|| black_box(ClassList::new()))
    });
    
    group.bench_function("add_single", |b| {
        b.iter(|| {
            let mut list = ClassList::new();
            list.add(black_box("btn"));
            black_box(list)
        })
    });
    
    group.bench_function("add_multiple", |b| {
        b.iter(|| {
            let mut list = ClassList::new();
            list.add(black_box("btn"));
            list.add(black_box("btn-primary"));
            list.add(black_box("btn-large"));
            black_box(list)
        })
    });
    
    group.bench_function("contains", |b| {
        let mut list = ClassList::new();
        list.add("btn");
        list.add("btn-primary");
        list.add("btn-large");
        
        b.iter(|| black_box(list.contains(black_box("btn-primary"))))
    });
    
    group.bench_function("toggle", |b| {
        let mut list = ClassList::new();
        list.add("btn");
        
        b.iter(|| {
            list.toggle(black_box("active"));
            black_box(&list);
        })
    });
    
    group.finish();
}

/// Simulate hit testing
fn bench_hit_testing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_testing");
    
    // Single widget hit test
    group.bench_function("single", |b| {
        let rect = Rect::new(100.0, 100.0, 200.0, 50.0);
        let point = Point::new(150.0, 120.0);
        
        b.iter(|| black_box(rect).contains(black_box(point)))
    });
    
    // Linear search through widgets
    for count in [10, 50, 100, 500] {
        group.bench_with_input(BenchmarkId::new("linear", count), &count, |b, &count| {
            let widgets: Vec<Rect> = (0..count)
                .map(|i| Rect::new((i % 10) as f32 * 120.0, (i / 10) as f32 * 60.0, 100.0, 50.0))
                .collect();
            let point = Point::new(550.0, 250.0);
            
            b.iter(|| {
                for widget in &widgets {
                    if widget.contains(black_box(point)) {
                        return black_box(Some(widget));
                    }
                }
                black_box(None)
            })
        });
    }
    
    group.finish();
}

/// Simulate color operations
fn bench_color_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("color");
    
    group.bench_function("from_rgb8", |b| {
        b.iter(|| black_box(Color::from_rgb8(black_box(59), black_box(130), black_box(246))))
    });
    
    group.bench_function("from_hex", |b| {
        b.iter(|| black_box(Color::from_hex(black_box("#3b82f6"))))
    });
    
    group.bench_function("to_rgba8", |b| {
        let color = Color::from_rgb8(59, 130, 246);
        b.iter(|| black_box(color).to_rgba8())
    });
    
    group.bench_function("to_rgba_f32", |b| {
        let color = Color::from_rgb8(59, 130, 246);
        b.iter(|| black_box(color).to_rgba_f32())
    });
    
    group.bench_function("blend", |b| {
        let color1 = Color::from_rgb8(59, 130, 246);
        let color2 = Color::from_rgba8(255, 255, 255, 128);
        b.iter(|| black_box(color1).blend(&black_box(color2)))
    });
    
    group.finish();
}

/// Simulate size calculations
fn bench_size_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("size");
    
    group.bench_function("new", |b| {
        b.iter(|| black_box(Size::new(black_box(100.0), black_box(50.0))))
    });
    
    group.bench_function("area", |b| {
        let s = Size::new(100.0, 50.0);
        b.iter(|| black_box(s.width * s.height))
    });
    
    group.bench_function("scale", |b| {
        let s = Size::new(100.0, 50.0);
        b.iter(|| black_box(Size::new(s.width * 2.0, s.height * 2.0)))
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    bench_widget_id_generation,
    bench_class_list,
    bench_hit_testing,
    bench_color_operations,
    bench_size_operations
);
criterion_main!(benches);
