//! OpenKit Design System Showcase
//!
//! A beautiful demonstration of OpenKit's Tailwind-inspired design system.

use openkit::prelude::*;
use openkit::widget::switch::ToggleSwitch;

fn main() {
    App::new()
        .title("OpenKit Design System")
        .size(900.0, 700.0)
        .theme(Theme::Auto)
        .load_css_file("examples/styles/openkit-design.css")
        .run(|| {
            // Main container with soft background
            Column::new()
                .gap(32.0)
                .padding(EdgeInsets::all(48.0))
                .class("demo-root")
                .child(
                    // Hero Header
                    col![16;
                        label!("OpenKit", class: "hero-title"),
                        label!("A modern, cross-platform UI toolkit for Rust", class: "hero-subtitle"),
                    ]
                )
                .child(
                    // Stats row
                    row![20;
                        stat_card("100%", "Pure Rust"),
                        stat_card("3", "Platforms"),
                        stat_card("CSS", "Styled"),
                        stat_card("GPU", "Accelerated"),
                    ]
                )
                .child(
                    // Button showcase in a card
                    Column::new()
                        .gap(20.0)
                        .padding(EdgeInsets::all(28.0))
                        .class("demo-section")
                        .child(label!("BUTTON VARIANTS", class: "section-title"))
                        .child(row![12;
                            button!("Primary", Primary, { println!("Primary!"); }),
                            button!("Secondary", Secondary, { println!("Secondary!"); }),
                            button!("Outline", Outline, { println!("Outline!"); }),
                            button!("Ghost", Ghost, { println!("Ghost!"); }),
                        ])
                        .child(row![12;
                            button!("Destructive", Destructive, { println!("Danger!"); }),
                        ])
                )
                .child(
                    // Controls section
                    Row::new()
                        .gap(24.0)
                        .child(
                            // Toggle switches card
                            Column::new()
                                .gap(20.0)
                                .padding(EdgeInsets::all(28.0))
                                .class("demo-section")
                                .child(label!("PREFERENCES", class: "section-title"))
                                .child(
                                    ToggleSwitch::new()
                                        .label("Dark Mode")
                                        .theme_toggle(true)
                                        .on_change(|enabled| {
                                            println!("Dark mode: {}", enabled);
                                        })
                                )
                                .child(
                                    switch!("Notifications", true, |enabled| {
                                        println!("Notifications: {}", enabled);
                                    })
                                )
                                .child(
                                    switch!("Auto-save", true, |enabled| {
                                        println!("Auto-save: {}", enabled);
                                    })
                                )
                                .child(
                                    switch!("Analytics", |enabled| {
                                        println!("Analytics: {}", enabled);
                                    })
                                )
                        )
                        .child(
                            // Input and badges card
                            Column::new()
                                .gap(20.0)
                                .padding(EdgeInsets::all(28.0))
                                .class("demo-section")
                                .child(label!("INPUT & BADGES", class: "section-title"))
                                .child(
                                    textfield!("Search...", class: "input")
                                )
                                .child(row![8;
                                    label!("New", class: "badge badge-primary"),
                                    label!("Active", class: "badge badge-success"),
                                    label!("Warning", class: "badge badge-warning"),
                                    label!("Error", class: "badge badge-destructive"),
                                ])
                                .child(label!("Beautiful badges for status indicators", class: "body-xs"))
                        )
                )
                .child(
                    // Footer
                    Row::new()
                        .gap(16.0)
                        .child(label!("Built with", class: "caption"))
                        .child(label!("Rust + Tailwind CSS Design", class: "caption text-primary"))
                )
        })
        .expect("Failed to run application");
}

/// Create a stat card widget
fn stat_card(value: &str, label_text: &str) -> impl Widget {
    Column::new()
        .gap(4.0)
        .padding(EdgeInsets::all(20.0))
        .class("stat-card")
        .child(Label::new(value).class("stat-value"))
        .child(Label::new(label_text).class("stat-label"))
}
