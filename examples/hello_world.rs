//! Hello World example for OpenKit.
//!
//! A beautiful, minimal OpenKit application showcasing the design system.

use openkit::prelude::*;

fn main() {
    App::new()
        .title("Hello OpenKit")
        .size(500.0, 400.0)
        .theme(Theme::Light)
        .load_css_file("examples/styles/openkit-design.css")
        .run(|| {
            // Centered card layout
            Column::new()
                .gap(0.0)
                .padding(EdgeInsets::all(48.0))
                .class("demo-root")
                .child(
                    Column::new()
                        .gap(24.0)
                        .padding(EdgeInsets::all(32.0))
                        .class("card")
                        .child(
                            col![8;
                                label!("Welcome to OpenKit", class: "heading"),
                                label!("A modern UI toolkit for Rust", class: "subtitle"),
                            ]
                        )
                        .child(
                            col![16;
                                button!("Get Started", Primary, {
                                    println!("Let's go!");
                                }),
                                button!("Learn More", Outline, {
                                    println!("Documentation...");
                                }),
                            ]
                        )
                        .child(
                            label!("Built with love in Rust", class: "caption")
                        )
                )
        })
        .expect("Failed to run application");
}
