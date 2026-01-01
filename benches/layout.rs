//! Layout engine benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use openkit::geometry::{Rect, Size, Point};
use openkit::layout::{Constraints, LayoutResult};

/// Simulate flexbox layout calculations
fn simulate_flex_layout(items: usize, available_width: f32) -> Vec<Rect> {
    let mut results = Vec::with_capacity(items);
    let item_width = available_width / items as f32;
    
    for i in 0..items {
        results.push(Rect::new(
            i as f32 * item_width,
            0.0,
            item_width,
            40.0,
        ));
    }
    
    results
}

/// Simulate nested layout calculations
fn simulate_nested_layout(depth: usize, children_per_level: usize) -> usize {
    let mut total_layouts = 0;
    
    fn layout_recursive(depth: usize, children: usize, total: &mut usize) {
        *total += 1;
        if depth > 0 {
            for _ in 0..children {
                layout_recursive(depth - 1, children, total);
            }
        }
    }
    
    layout_recursive(depth, children_per_level, &mut total_layouts);
    total_layouts
}

/// Simulate constraint solving
fn simulate_constraint_solving(constraints: &Constraints, preferred: Size) -> Size {
    Size::new(
        preferred.width.clamp(constraints.min_width, constraints.max_width),
        preferred.height.clamp(constraints.min_height, constraints.max_height),
    )
}

fn bench_flex_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("flex_layout");
    
    for items in [5, 10, 20, 50, 100] {
        group.bench_with_input(BenchmarkId::new("items", items), &items, |b, &items| {
            b.iter(|| simulate_flex_layout(black_box(items), black_box(1280.0)))
        });
    }
    
    group.finish();
}

fn bench_nested_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_layout");
    
    // Different depths with 3 children per level
    for depth in [2, 3, 4, 5] {
        let total = simulate_nested_layout(depth, 3);
        group.bench_with_input(
            BenchmarkId::new("depth", format!("{} ({})", depth, total)), 
            &depth, 
            |b, &depth| {
                b.iter(|| simulate_nested_layout(black_box(depth), black_box(3)))
            }
        );
    }
    
    group.finish();
}

fn bench_constraint_solving(c: &mut Criterion) {
    let mut group = c.benchmark_group("constraint_solving");
    
    let constraints = Constraints {
        min_width: 100.0,
        max_width: 800.0,
        min_height: 50.0,
        max_height: 600.0,
    };
    
    let preferred = Size::new(400.0, 300.0);
    
    group.bench_function("single", |b| {
        b.iter(|| simulate_constraint_solving(black_box(&constraints), black_box(preferred)))
    });
    
    // Batch constraint solving
    group.bench_function("batch_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(simulate_constraint_solving(&constraints, preferred));
            }
        })
    });
    
    group.finish();
}

fn bench_rect_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("rect_operations");
    
    let rect = Rect::new(100.0, 100.0, 200.0, 150.0);
    let point = Point::new(150.0, 120.0);
    
    group.bench_function("contains_point", |b| {
        b.iter(|| black_box(rect).contains(black_box(point)))
    });
    
    let other = Rect::new(150.0, 120.0, 100.0, 80.0);
    group.bench_function("intersects", |b| {
        b.iter(|| black_box(rect).intersects(&black_box(other)))
    });
    
    group.bench_function("union", |b| {
        b.iter(|| black_box(rect).union(&black_box(other)))
    });
    
    group.finish();
}

criterion_group!(benches, bench_flex_layout, bench_nested_layout, bench_constraint_solving, bench_rect_operations);
criterion_main!(benches);
