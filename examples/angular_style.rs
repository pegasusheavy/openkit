//! Angular-style component example.
//!
//! Demonstrates the Angular-like component system with:
//! - Structural directives (ng_if, ng_for, ng_switch)
//! - Property binding
//! - Event binding
//! - Pipes

use openkit::prelude::*;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

fn main() {
    // Use Arc<AtomicI32> for thread-safe state that can be shared across closures
    let count = Arc::new(AtomicI32::new(0));
    let items = ["Apple", "Banana", "Cherry", "Date"];
    let show_list = true;
    let status = "ready";

    // Clone for closures
    let count_for_display = count.clone();
    let count_for_inc = count.clone();
    let count_for_dec = count.clone();

    App::new()
        .title("Angular-Style Components")
        .size(800.0, 600.0)
        .theme(Theme::Auto)
        .run(move || {
            let current_count = count_for_display.load(Ordering::SeqCst);
            let count_inc = count_for_inc.clone();
            let count_dec = count_for_dec.clone();

            col![24;
                // Header
                label!("Angular-Style Component Demo", class: "title"),

                // Counter component
                label!("Counter with Atomic State:"),
                col![8;
                    label!(format!("Count: {}", current_count)),
                    row![8;
                        Button::new("−").on_click(move || {
                            count_dec.fetch_sub(1, Ordering::SeqCst);
                            println!("Decremented");
                        }),
                        Button::new("+").on_click(move || {
                            count_inc.fetch_add(1, Ordering::SeqCst);
                            println!("Incremented");
                        }),
                    ],
                ],

                // ng_if - Conditional rendering
                label!("Conditional Rendering (If directive):"),
                {
                    // Using the If directive
                    If::new(show_list)
                        .then(label!("✓ List is visible"))
                        .otherwise(label!("✗ List is hidden"))
                        .render()
                        .unwrap_or_else(|| label!(""))
                },

                // ng_for - List rendering
                label!("List Rendering (For directive):"),
                col![4;
                    // Using the For directive
                    Label::new(format!("• {}", items[0])),
                    Label::new(format!("• {}", items[1])),
                    Label::new(format!("• {}", items[2])),
                    Label::new(format!("• {}", items[3])),
                ],

                // ng_switch - Switch rendering
                label!("Switch Rendering (Switch directive):"),
                {
                    // Using the Switch directive
                    Switch::on(status)
                        .case("loading", label!("⏳ Loading..."))
                        .case("error", label!("❌ Error occurred"))
                        .case("ready", label!("✅ Ready to go!"))
                        .default(label!("❓ Unknown status"))
                        .render()
                        .unwrap_or_else(|| label!(""))
                },

                // Button variants with event binding
                label!("Event Binding with Button Variants:"),
                row![8;
                    button!("Primary", { println!("Primary clicked"); }),
                    button!("Secondary", Secondary, { println!("Secondary clicked"); }),
                    button!("Outline", Outline, { println!("Outline clicked"); }),
                    button!("Destructive", Destructive, { println!("Destructive clicked"); }),
                ],

                // Text field with change event
                label!("Two-way Binding Simulation:"),
                textfield!("Type here...", |value| {
                    println!("Input changed: {}", value);
                }),

                // Pipes demonstration
                label!("Pipe Transformations:"),
                {
                    let text = "hello world";
                    let upper = UppercasePipe.transform(text);
                    let lower = LowercasePipe.transform(&upper);
                    label!(format!("Original: '{}' → Upper: '{}' → Lower: '{}'", text, upper, lower))
                },
            ]
        })
        .expect("Failed to run application");
}
