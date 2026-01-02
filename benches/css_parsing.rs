//! CSS parsing benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use openkit::css::StyleManager;

/// Simple CSS with a few rules
const SIMPLE_CSS: &str = r#"
.button {
    background: #3b82f6;
    color: white;
    padding: 8px 16px;
    border-radius: 4px;
}

.button:hover {
    background: #2563eb;
}
"#;

/// Medium CSS with more rules
const MEDIUM_CSS: &str = r#"
:root {
    --primary: #3b82f6;
    --secondary: #6b7280;
    --success: #10b981;
    --warning: #f59e0b;
    --danger: #ef4444;
    --background: #0f172a;
    --surface: #1e293b;
    --text: #f1f5f9;
}

.btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px 16px;
    border-radius: 6px;
    font-weight: 500;
    transition: all 0.15s ease;
}

.btn-primary {
    background: var(--primary);
    color: white;
}

.btn-primary:hover {
    background: #2563eb;
    transform: translateY(-1px);
}

.btn-secondary {
    background: var(--secondary);
    color: white;
}

.card {
    background: var(--surface);
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.card-header {
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: 16px;
}

.card-body {
    color: var(--text);
}

.input {
    background: var(--background);
    border: 1px solid var(--secondary);
    border-radius: 6px;
    padding: 8px 12px;
    color: var(--text);
}

.input:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.3);
}
"#;

/// Complex CSS with many rules (simulating a full theme)
fn generate_complex_css() -> String {
    let mut css = String::with_capacity(50000);
    
    // Variables
    css.push_str(":root {\n");
    for i in 0..50 {
        css.push_str(&format!("    --color-{}: #{};\n", i, format!("{:06x}", i * 12345 % 0xFFFFFF)));
    }
    css.push_str("}\n\n");
    
    // Many widget classes
    let widgets = ["button", "input", "card", "label", "container", "row", "column", 
                   "header", "footer", "sidebar", "nav", "menu", "dropdown", "modal",
                   "tooltip", "badge", "avatar", "progress", "slider", "switch"];
    
    for widget in widgets {
        css.push_str(&format!(r#"
.{widget} {{
    display: flex;
    padding: 8px;
    margin: 4px;
    border-radius: 4px;
}}

.{widget}:hover {{
    background: rgba(255, 255, 255, 0.1);
}}

.{widget}:active {{
    background: rgba(255, 255, 255, 0.2);
}}

.{widget}--primary {{
    background: var(--color-1);
    color: white;
}}

.{widget}--secondary {{
    background: var(--color-2);
    color: white;
}}

.{widget}--large {{
    padding: 16px 24px;
    font-size: 18px;
}}

.{widget}--small {{
    padding: 4px 8px;
    font-size: 12px;
}}
"#));
    }
    
    css
}

fn bench_css_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("css_parsing");
    
    // Simple CSS
    group.throughput(Throughput::Bytes(SIMPLE_CSS.len() as u64));
    group.bench_with_input(BenchmarkId::new("simple", SIMPLE_CSS.len()), SIMPLE_CSS, |b, css| {
        b.iter(|| {
            let mut manager = StyleManager::new();
            manager.load_css(black_box(css)).ok()
        })
    });
    
    // Medium CSS
    group.throughput(Throughput::Bytes(MEDIUM_CSS.len() as u64));
    group.bench_with_input(BenchmarkId::new("medium", MEDIUM_CSS.len()), MEDIUM_CSS, |b, css| {
        b.iter(|| {
            let mut manager = StyleManager::new();
            manager.load_css(black_box(css)).ok()
        })
    });
    
    // Complex CSS
    let complex_css = generate_complex_css();
    group.throughput(Throughput::Bytes(complex_css.len() as u64));
    group.bench_with_input(BenchmarkId::new("complex", complex_css.len()), &complex_css, |b, css| {
        b.iter(|| {
            let mut manager = StyleManager::new();
            manager.load_css(black_box(css)).ok()
        })
    });
    
    group.finish();
}

fn bench_style_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("style_manager");
    
    group.bench_function("create_new", |b| {
        b.iter(|| {
            black_box(StyleManager::new())
        })
    });
    
    group.bench_function("create_empty", |b| {
        b.iter(|| {
            black_box(StyleManager::empty())
        })
    });
    
    // Load and access
    group.bench_function("load_and_access", |b| {
        b.iter(|| {
            let mut manager = StyleManager::new();
            manager.load_css(black_box(SIMPLE_CSS)).ok();
            black_box(manager)
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_css_parsing, bench_style_manager);
criterion_main!(benches);
