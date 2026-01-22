//! Widget Showcase - A comprehensive demo of all OpenKit widgets
//!
//! Run with: cargo run --example widget_showcase

use openkit::prelude::*;
use openkit::widget::switch::ToggleSwitch;

fn main() {
    App::new()
        .title("OpenKit Widget Showcase")
        .size(1200.0, 1400.0)
        .theme(Theme::Light)
        .load_css_file("examples/styles/openkit-design.css")
        .run(|| {
            Column::new()
                .gap(32.0)
                .padding(EdgeInsets::all(40.0))
                .class("demo-root")
                // Header
                .child(header_section())
                // Buttons Section
                .child(buttons_section())
                // Input Controls Section
                .child(input_controls_section())
                // Selection Controls Section
                .child(selection_controls_section())
                // Progress & Loading Section
                .child(progress_section())
                // Avatar & Cards Section
                .child(cards_section())
                // Navigation Section
                .child(navigation_section())
                // Footer
                .child(footer_section())
        })
        .expect("Failed to run application");
}

/// Header section with title and description
fn header_section() -> impl Widget {
    Column::new()
        .gap(12.0)
        .child(label!("OpenKit Widget Showcase", class: "hero-title"))
        .child(label!("A comprehensive demonstration of all available widgets", class: "hero-subtitle"))
        .child(
            Row::new()
                .gap(16.0)
                .child(label!("v0.1.2", class: "badge badge-primary"))
                .child(label!("Pure Rust", class: "badge badge-success"))
                .child(label!("Cross-Platform", class: "badge badge-warning"))
        )
}

/// Button variants section
fn buttons_section() -> impl Widget {
    Card::new()
        .padding(28.0)
        .child(
            Column::new()
                .gap(20.0)
                .child(label!("BUTTONS", class: "section-title"))
                .child(label!("Various button styles for different actions", class: "subtitle"))
                // Standard variants
                .child(
                    Row::new()
                        .gap(12.0)
                        .child(button!("Primary", Primary, { println!("Primary clicked"); }))
                        .child(button!("Secondary", Secondary, { println!("Secondary clicked"); }))
                        .child(button!("Outline", Outline, { println!("Outline clicked"); }))
                        .child(button!("Ghost", Ghost, { println!("Ghost clicked"); }))
                        .child(button!("Destructive", Destructive, { println!("Destructive clicked"); }))
                )
                // Separator
                .child(Separator::horizontal())
                // Icon Buttons
                .child(label!("ICON BUTTONS", class: "section-title"))
                .child(
                    Row::new()
                        .gap(12.0)
                        .child(
                            IconButton::new("+")
                                .tooltip("Add item")
                                .variant(IconButtonVariant::Filled)
                                .on_click(|| println!("Add clicked"))
                        )
                        .child(
                            IconButton::new("-")
                                .tooltip("Remove item")
                                .variant(IconButtonVariant::Outline)
                                .on_click(|| println!("Remove clicked"))
                        )
                        .child(
                            IconButton::new("X")
                                .tooltip("Close")
                                .variant(IconButtonVariant::Destructive)
                                .on_click(|| println!("Close clicked"))
                        )
                        .child(
                            IconButton::new("?")
                                .tooltip("Help")
                                .variant(IconButtonVariant::Ghost)
                                .on_click(|| println!("Help clicked"))
                        )
                )
        )
}

/// Input controls section
fn input_controls_section() -> impl Widget {
    Card::new()
        .padding(28.0)
        .child(
            Column::new()
                .gap(20.0)
                .child(label!("INPUT CONTROLS", class: "section-title"))
                .child(label!("Text fields, password inputs, and sliders", class: "subtitle"))
                // Text inputs row
                .child(
                    Row::new()
                        .gap(16.0)
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(label!("Text Field", class: "body-xs"))
                                .child(
                                    textfield!("Enter your name...", |text| {
                                        println!("Name: {}", text);
                                    })
                                )
                        )
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(label!("Email Field", class: "body-xs"))
                                .child(
                                    textfield!("email@example.com", |text| {
                                        println!("Email: {}", text);
                                    })
                                )
                        )
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(label!("Password", class: "body-xs"))
                                .child(
                                    PasswordField::new()
                                        .placeholder("Enter password...")
                                        .on_change(|_| println!("Password changed"))
                                )
                        )
                )
                .child(Separator::horizontal())
                // Sliders
                .child(label!("SLIDERS", class: "section-title"))
                .child(
                    Row::new()
                        .gap(24.0)
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(label!("Volume", class: "body-xs"))
                                .child(
                                    Slider::new()
                                        .min(0.0)
                                        .max(100.0)
                                        .value(50.0)
                                        .show_value(true)
                                        .on_change(|v| println!("Volume: {:.0}%", v))
                                )
                        )
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(label!("Brightness", class: "body-xs"))
                                .child(
                                    Slider::new()
                                        .min(0.0)
                                        .max(100.0)
                                        .value(75.0)
                                        .step(10.0)
                                        .show_value(true)
                                        .on_change(|v| println!("Brightness: {:.0}%", v))
                                )
                        )
                )
        )
}

/// Selection controls section
fn selection_controls_section() -> impl Widget {
    Card::new()
        .padding(28.0)
        .child(
            Column::new()
                .gap(20.0)
                .child(label!("SELECTION CONTROLS", class: "section-title"))
                .child(label!("Checkboxes, toggles, and dropdowns", class: "subtitle"))
                .child(
                    Row::new()
                        .gap(32.0)
                        // Checkboxes column
                        .child(
                            Column::new()
                                .gap(12.0)
                                .child(label!("Checkboxes", class: "body-xs"))
                                .child(
                                    Checkbox::new()
                                        .label("Enable notifications")
                                        .checked(true)
                                        .on_change(|checked| println!("Notifications: {}", checked))
                                )
                                .child(
                                    Checkbox::new()
                                        .label("Auto-save documents")
                                        .on_change(|checked| println!("Auto-save: {}", checked))
                                )
                                .child(
                                    Checkbox::new()
                                        .label("Show hidden files")
                                        .on_change(|checked| println!("Hidden files: {}", checked))
                                )
                        )
                        // Toggle switches column
                        .child(
                            Column::new()
                                .gap(12.0)
                                .child(label!("Toggle Switches", class: "body-xs"))
                                .child(
                                    ToggleSwitch::new()
                                        .label("Dark Mode")
                                        .theme_toggle(true)
                                        .on_change(|enabled| println!("Dark mode: {}", enabled))
                                )
                                .child(
                                    ToggleSwitch::new()
                                        .label("Airplane Mode")
                                        .on_change(|enabled| println!("Airplane mode: {}", enabled))
                                )
                                .child(
                                    ToggleSwitch::new()
                                        .label("WiFi")
                                        .checked(true)
                                        .on_change(|enabled| println!("WiFi: {}", enabled))
                                )
                        )
                        // Dropdown column
                        .child(
                            Column::new()
                                .gap(12.0)
                                .child(label!("Dropdown", class: "body-xs"))
                                .child(
                                    Dropdown::new()
                                        .placeholder("Select language...")
                                        .option(DropdownOption::new("en", "English"))
                                        .option(DropdownOption::new("es", "Spanish"))
                                        .option(DropdownOption::new("fr", "French"))
                                        .option(DropdownOption::new("de", "German"))
                                        .option(DropdownOption::new("ja", "Japanese"))
                                        .on_change(|id| println!("Selected: {}", id))
                                )
                        )
                )
        )
}

/// Progress and loading indicators section
fn progress_section() -> impl Widget {
    Card::new()
        .padding(28.0)
        .child(
            Column::new()
                .gap(20.0)
                .child(label!("PROGRESS & LOADING", class: "section-title"))
                .child(label!("Progress bars and spinners", class: "subtitle"))
                // Progress bars
                .child(
                    Column::new()
                        .gap(16.0)
                        .child(
                            Row::new()
                                .gap(16.0)
                                .child(label!("Download:", class: "body-xs"))
                                .child(
                                    Progress::new()
                                        .value(0.75)
                                        .size(ProgressSize::Medium)
                                        .show_label(true)
                                )
                        )
                        .child(
                            Row::new()
                                .gap(16.0)
                                .child(label!("Upload:", class: "body-xs"))
                                .child(
                                    Progress::new()
                                        .value(0.45)
                                        .size(ProgressSize::Medium)
                                        .variant(ProgressVariant::Striped)
                                        .show_label(true)
                                )
                        )
                        .child(
                            Row::new()
                                .gap(16.0)
                                .child(label!("Processing:", class: "body-xs"))
                                .child(
                                    Progress::new()
                                        .size(ProgressSize::Small)
                                        .variant(ProgressVariant::Indeterminate)
                                )
                        )
                )
                .child(Separator::horizontal())
                // Spinners
                .child(label!("SPINNERS", class: "section-title"))
                .child(
                    Row::new()
                        .gap(24.0)
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(Spinner::new().size(SpinnerSize::Small))
                                .child(label!("Small", class: "caption"))
                        )
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(Spinner::new().size(SpinnerSize::Medium))
                                .child(label!("Medium", class: "caption"))
                        )
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(Spinner::new().size(SpinnerSize::Large))
                                .child(label!("Large", class: "caption"))
                        )
                        .child(
                            Column::new()
                                .gap(8.0)
                                .child(Spinner::new().size(SpinnerSize::XLarge))
                                .child(label!("X-Large", class: "caption"))
                        )
                )
        )
}

/// Cards and avatars section
fn cards_section() -> impl Widget {
    Card::new()
        .padding(28.0)
        .child(
            Column::new()
                .gap(20.0)
                .child(label!("AVATARS & CARDS", class: "section-title"))
                .child(label!("User representations and content containers", class: "subtitle"))
                // Avatars row
                .child(
                    Row::new()
                        .gap(16.0)
                        .child(
                            Avatar::new()
                                .initials("JD")
                                .size(AvatarSize::XSmall)
                        )
                        .child(
                            Avatar::new()
                                .initials("AB")
                                .size(AvatarSize::Small)
                        )
                        .child(
                            Avatar::new()
                                .initials("CD")
                                .size(AvatarSize::Medium)
                        )
                        .child(
                            Avatar::new()
                                .initials("EF")
                                .size(AvatarSize::Large)
                        )
                        .child(
                            Avatar::new()
                                .initials("GH")
                                .size(AvatarSize::XLarge)
                                .shape(AvatarShape::Rounded)
                        )
                )
                .child(Separator::horizontal())
                // Card variants
                .child(label!("CARD VARIANTS", class: "section-title"))
                .child(
                    Row::new()
                        .gap(16.0)
                        .child(
                            Card::new()
                                .variant(CardVariant::Default)
                                .padding(16.0)
                                .child(
                                    Column::new()
                                        .gap(8.0)
                                        .child(label!("Default", class: "body-xs"))
                                        .child(label!("Standard card", class: "caption"))
                                )
                        )
                        .child(
                            Card::new()
                                .variant(CardVariant::Elevated)
                                .padding(16.0)
                                .child(
                                    Column::new()
                                        .gap(8.0)
                                        .child(label!("Elevated", class: "body-xs"))
                                        .child(label!("More shadow", class: "caption"))
                                )
                        )
                        .child(
                            Card::new()
                                .variant(CardVariant::Outlined)
                                .padding(16.0)
                                .child(
                                    Column::new()
                                        .gap(8.0)
                                        .child(label!("Outlined", class: "body-xs"))
                                        .child(label!("Border only", class: "caption"))
                                )
                        )
                        .child(
                            Card::new()
                                .variant(CardVariant::Ghost)
                                .padding(16.0)
                                .child(
                                    Column::new()
                                        .gap(8.0)
                                        .child(label!("Ghost", class: "body-xs"))
                                        .child(label!("Minimal", class: "caption"))
                                )
                        )
                )
        )
}

/// List and separators section (replacing tabs due to layout bug)
fn navigation_section() -> impl Widget {
    Card::new()
        .padding(28.0)
        .child(
            Column::new()
                .gap(20.0)
                .child(label!("LISTS & SEPARATORS", class: "section-title"))
                .child(label!("Visual division and list components", class: "subtitle"))
                // Separators
                .child(
                    Column::new()
                        .gap(12.0)
                        .child(label!("Horizontal Separators", class: "body-xs"))
                        .child(Separator::horizontal())
                        .child(label!("Content between separators", class: "caption"))
                        .child(Separator::horizontal().thickness(2.0))
                        .child(label!("Thicker separator above", class: "caption"))
                )
                .child(Separator::horizontal())
                // List example using cards
                .child(label!("SIMPLE LIST", class: "section-title"))
                .child(
                    Column::new()
                        .gap(8.0)
                        .child(list_item("Settings", "Configure application preferences"))
                        .child(list_item("Profile", "Manage your account details"))
                        .child(list_item("Security", "Update passwords and 2FA"))
                        .child(list_item("Notifications", "Control alert preferences"))
                )
        )
}

/// Helper to create a simple list item
fn list_item(title: &str, description: &str) -> impl Widget {
    Card::new()
        .variant(CardVariant::Ghost)
        .padding(12.0)
        .child(
            Column::new()
                .gap(4.0)
                .child(Label::new(title).class("body-xs"))
                .child(Label::new(description).class("caption"))
        )
}

/// Footer section
fn footer_section() -> impl Widget {
    Column::new()
        .gap(12.0)
        .child(Separator::horizontal())
        .child(
            Row::new()
                .gap(16.0)
                .child(label!("Built with", class: "caption"))
                .child(label!("OpenKit", class: "caption text-primary"))
                .child(label!("|", class: "caption"))
                .child(label!("Pure Rust", class: "caption"))
                .child(label!("|", class: "caption"))
                .child(label!("CSS Styled", class: "caption"))
        )
        .child(
            label!("Windows | macOS | Linux", class: "caption")
        )
}
