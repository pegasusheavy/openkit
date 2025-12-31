#![allow(clippy::type_complexity)]
//! Angular-inspired component system for OpenKit.
//!
//! This module provides a component-based architecture similar to Angular:
//!
//! - **Components** with templates, styles, and logic
//! - **Property binding** with `prop()` and `bind()`
//! - **Event binding** with `on()` and `emit()`
//! - **Lifecycle hooks** (`on_init`, `on_destroy`, `on_changes`)
//! - **Structural directives** (`If`, `For`, `Switch`)
//! - **Two-way binding** with `model()`
//!
//! # Example
//!
//! ```rust,ignore
//! use openkit::prelude::*;
//! use openkit::component::*;
//!
//! // Define a component
//! let counter = Component::new("counter")
//!     .state(0i32)
//!     .template(|state, ctx| {
//!         col![16;
//!             label!(format!("Count: {}", state)),
//!             row![8;
//!                 button!("-", { ctx.update(|s| *s -= 1); }),
//!                 button!("+", { ctx.update(|s| *s += 1); }),
//!             ],
//!         ]
//!     })
//!     .on_init(|state| println!("Counter initialized with {}", state))
//!     .build();
//! ```

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

/// Component lifecycle hooks.
pub trait Lifecycle {
    /// Called when the component is initialized.
    fn on_init(&mut self) {}

    /// Called when inputs change.
    fn on_changes(&mut self, _changes: &Changes) {}

    /// Called after the view is initialized.
    fn on_view_init(&mut self) {}

    /// Called on each update cycle.
    fn on_check(&mut self) {}

    /// Called when the component is destroyed.
    fn on_destroy(&mut self) {}
}

/// Represents changes to component inputs.
#[derive(Debug, Default)]
pub struct Changes {
    changed: HashMap<String, (Box<dyn Any>, Box<dyn Any>)>,
}

impl Changes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a specific input changed.
    pub fn has(&self, name: &str) -> bool {
        self.changed.contains_key(name)
    }

    /// Get the previous and current values for a changed input.
    pub fn get<T: 'static + Clone>(&self, name: &str) -> Option<(T, T)> {
        self.changed.get(name).and_then(|(prev, curr)| {
            let prev = prev.downcast_ref::<T>()?;
            let curr = curr.downcast_ref::<T>()?;
            Some((prev.clone(), curr.clone()))
        })
    }

    /// Check if this is the first change (initialization).
    pub fn is_first_change(&self, name: &str) -> bool {
        // First change has no previous value
        self.changed.get(name).map(|(p, _)| p.is::<()>()).unwrap_or(false)
    }
}

/// A reactive state container.
#[derive(Clone)]
pub struct State<T> {
    inner: Rc<RefCell<T>>,
    subscribers: Rc<RefCell<Vec<Box<dyn Fn(&T)>>>>,
}

impl<T: 'static> State<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
            subscribers: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get the current value.
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        self.inner.borrow()
    }

    /// Set a new value and notify subscribers.
    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value;
        self.notify();
    }

    /// Update the value with a function.
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        f(&mut self.inner.borrow_mut());
        self.notify();
    }

    /// Subscribe to value changes.
    pub fn subscribe<F: Fn(&T) + 'static>(&self, f: F) {
        self.subscribers.borrow_mut().push(Box::new(f));
    }

    fn notify(&self) {
        let value = self.inner.borrow();
        for subscriber in self.subscribers.borrow().iter() {
            subscriber(&*value);
        }
    }
}

impl<T: Clone + 'static> State<T> {
    /// Get a clone of the current value.
    pub fn value(&self) -> T {
        self.inner.borrow().clone()
    }
}

impl<T: Default + 'static> Default for State<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Event emitter for component outputs.
#[derive(Clone)]
pub struct EventEmitter<T> {
    handlers: Rc<RefCell<Vec<Box<dyn Fn(T)>>>>,
}

impl<T: Clone + 'static> EventEmitter<T> {
    pub fn new() -> Self {
        Self {
            handlers: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Emit an event to all subscribers.
    pub fn emit(&self, value: T) {
        for handler in self.handlers.borrow().iter() {
            handler(value.clone());
        }
    }

    /// Subscribe to this event.
    pub fn subscribe<F: Fn(T) + 'static>(&self, handler: F) {
        self.handlers.borrow_mut().push(Box::new(handler));
    }
}

impl<T: Clone + 'static> Default for EventEmitter<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Component context for template rendering.
pub struct ComponentContext<S> {
    state: State<S>,
    should_update: Rc<RefCell<bool>>,
}

impl<S: 'static> ComponentContext<S> {
    pub fn new(state: State<S>) -> Self {
        Self {
            state,
            should_update: Rc::new(RefCell::new(false)),
        }
    }

    /// Get the current state.
    pub fn state(&self) -> std::cell::Ref<'_, S> {
        self.state.get()
    }

    /// Update the state.
    pub fn update<F: FnOnce(&mut S)>(&self, f: F) {
        self.state.update(f);
        *self.should_update.borrow_mut() = true;
    }

    /// Set the state to a new value.
    pub fn set(&self, value: S) {
        self.state.set(value);
        *self.should_update.borrow_mut() = true;
    }
}

impl<S: Clone + 'static> ComponentContext<S> {
    /// Get a clone of the current state value.
    pub fn value(&self) -> S {
        self.state.value()
    }
}

/// Component builder with Angular-like API.
pub struct ComponentBuilder<S, W> {
    name: String,
    state: Option<S>,
    template: Option<Box<dyn Fn(&S, &ComponentContext<S>) -> W>>,
    styles: Vec<String>,
    on_init: Option<Box<dyn Fn(&S)>>,
    on_destroy: Option<Box<dyn Fn(&S)>>,
    on_changes: Option<Box<dyn Fn(&S, &Changes)>>,
    inputs: HashMap<String, Box<dyn Any>>,
    _marker: PhantomData<W>,
}

impl<S: Default + 'static, W: 'static> ComponentBuilder<S, W> {
    /// Create a new component builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: None,
            template: None,
            styles: Vec::new(),
            on_init: None,
            on_destroy: None,
            on_changes: None,
            inputs: HashMap::new(),
            _marker: PhantomData,
        }
    }
}

impl<S: 'static, W: 'static> ComponentBuilder<S, W> {
    /// Set the initial state.
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    /// Set the template function.
    pub fn template<F>(mut self, f: F) -> Self
    where
        F: Fn(&S, &ComponentContext<S>) -> W + 'static,
    {
        self.template = Some(Box::new(f));
        self
    }

    /// Add component styles (CSS).
    pub fn styles(mut self, css: impl Into<String>) -> Self {
        self.styles.push(css.into());
        self
    }

    /// Set the on_init lifecycle hook.
    pub fn on_init<F: Fn(&S) + 'static>(mut self, f: F) -> Self {
        self.on_init = Some(Box::new(f));
        self
    }

    /// Set the on_destroy lifecycle hook.
    pub fn on_destroy<F: Fn(&S) + 'static>(mut self, f: F) -> Self {
        self.on_destroy = Some(Box::new(f));
        self
    }

    /// Set the on_changes lifecycle hook.
    pub fn on_changes<F: Fn(&S, &Changes) + 'static>(mut self, f: F) -> Self {
        self.on_changes = Some(Box::new(f));
        self
    }

    /// Define an input property.
    pub fn input<T: 'static>(mut self, name: &str, default: T) -> Self {
        self.inputs.insert(name.to_string(), Box::new(default));
        self
    }
}

impl<S: Clone + 'static, W: crate::widget::Widget + 'static> ComponentBuilder<S, W> {
    /// Build the component.
    pub fn build(self) -> BuiltComponent<S, W> {
        let state = State::new(self.state.expect("Component requires state"));

        // Call on_init if provided
        if let Some(on_init) = &self.on_init {
            on_init(&state.get());
        }

        BuiltComponent {
            name: self.name,
            state,
            template: self.template.expect("Component requires template"),
            styles: self.styles,
            on_destroy: self.on_destroy,
            _marker: PhantomData,
        }
    }
}

/// A built component ready to render.
pub struct BuiltComponent<S: 'static, W> {
    name: String,
    state: State<S>,
    template: Box<dyn Fn(&S, &ComponentContext<S>) -> W>,
    styles: Vec<String>,
    on_destroy: Option<Box<dyn Fn(&S)>>,
    _marker: PhantomData<W>,
}

impl<S: Clone + 'static, W: crate::widget::Widget + 'static> BuiltComponent<S, W> {
    /// Render the component.
    pub fn render(&self) -> W {
        let ctx = ComponentContext::new(self.state.clone());
        (self.template)(&self.state.get(), &ctx)
    }

    /// Get the component name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the component styles.
    pub fn styles(&self) -> &[String] {
        &self.styles
    }
}

impl<S: 'static, W> Drop for BuiltComponent<S, W> {
    fn drop(&mut self) {
        if let Some(on_destroy) = &self.on_destroy {
            on_destroy(&self.state.get());
        }
    }
}

/// Shorthand for creating a component.
pub fn component<S: Default + 'static, W: 'static>(name: &str) -> ComponentBuilder<S, W> {
    ComponentBuilder::new(name)
}

// ============================================================================
// Structural Directives (Angular-like *ngIf, *ngFor, *ngSwitch)
// ============================================================================

/// Conditional rendering directive (like *ngIf).
pub struct If<W> {
    condition: bool,
    then_widget: Option<W>,
    else_widget: Option<W>,
}

impl<W> If<W> {
    /// Create a conditional directive.
    pub fn new(condition: bool) -> Self {
        Self {
            condition,
            then_widget: None,
            else_widget: None,
        }
    }

    /// Set the widget to show when condition is true.
    pub fn then(mut self, widget: W) -> Self {
        self.then_widget = Some(widget);
        self
    }

    /// Set the widget to show when condition is false.
    pub fn otherwise(mut self, widget: W) -> Self {
        self.else_widget = Some(widget);
        self
    }

    /// Render the appropriate widget.
    pub fn render(self) -> Option<W> {
        if self.condition {
            self.then_widget
        } else {
            self.else_widget
        }
    }
}

/// List rendering directive (like *ngFor).
pub struct For<T, W, F>
where
    F: Fn(T, usize) -> W,
{
    items: Vec<T>,
    template: F,
    _marker: PhantomData<W>,
}

impl<T, W, F> For<T, W, F>
where
    F: Fn(T, usize) -> W,
{
    /// Create a for directive.
    pub fn each(items: impl IntoIterator<Item = T>, template: F) -> Self {
        Self {
            items: items.into_iter().collect(),
            template,
            _marker: PhantomData,
        }
    }

    /// Render all items.
    pub fn render(self) -> Vec<W> {
        self.items
            .into_iter()
            .enumerate()
            .map(|(i, item)| (self.template)(item, i))
            .collect()
    }
}

/// Switch/case directive (like *ngSwitch).
pub struct Switch<T, W> {
    value: T,
    cases: Vec<(T, W)>,
    default: Option<W>,
}

impl<T: PartialEq, W> Switch<T, W> {
    /// Create a switch directive.
    pub fn on(value: T) -> Self {
        Self {
            value,
            cases: Vec::new(),
            default: None,
        }
    }

    /// Add a case.
    pub fn case(mut self, match_value: T, widget: W) -> Self {
        self.cases.push((match_value, widget));
        self
    }

    /// Set the default case.
    pub fn default(mut self, widget: W) -> Self {
        self.default = Some(widget);
        self
    }

    /// Render the matching case.
    pub fn render(self) -> Option<W> {
        for (case_value, widget) in self.cases {
            if case_value == self.value {
                return Some(widget);
            }
        }
        self.default
    }
}

// ============================================================================
// Property & Event Binding Helpers
// ============================================================================

/// Property binding wrapper.
#[derive(Clone)]
pub struct Binding<T> {
    value: Rc<RefCell<T>>,
    on_change: Rc<RefCell<Option<Box<dyn Fn(&T)>>>>,
}

impl<T: 'static> Binding<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            on_change: Rc::new(RefCell::new(None)),
        }
    }

    /// Get the current value.
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        self.value.borrow()
    }

    /// Set the value and trigger change callback.
    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
        if let Some(callback) = self.on_change.borrow().as_ref() {
            callback(&*self.value.borrow());
        }
    }

    /// Set change callback (for two-way binding).
    pub fn on_change<F: Fn(&T) + 'static>(&self, f: F) {
        *self.on_change.borrow_mut() = Some(Box::new(f));
    }
}

impl<T: Clone + 'static> Binding<T> {
    pub fn value(&self) -> T {
        self.value.borrow().clone()
    }
}

impl<T: Default + 'static> Default for Binding<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Two-way binding model (like [(ngModel)]).
pub struct Model<T> {
    binding: Binding<T>,
}

impl<T: 'static> Model<T> {
    pub fn new(value: T) -> Self {
        Self {
            binding: Binding::new(value),
        }
    }

    /// Get the binding for property access.
    pub fn binding(&self) -> &Binding<T> {
        &self.binding
    }

    /// Get current value.
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        self.binding.get()
    }

    /// Set value.
    pub fn set(&self, value: T) {
        self.binding.set(value);
    }
}

impl<T: Clone + 'static> Model<T> {
    pub fn value(&self) -> T {
        self.binding.value()
    }
}

impl<T: Default + 'static> Default for Model<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

// ============================================================================
// Pipe-like Transformations
// ============================================================================

/// Trait for Angular-like pipes.
pub trait Pipe<T, U> {
    fn transform(&self, value: T) -> U;
}

/// Uppercase pipe.
pub struct UppercasePipe;

impl Pipe<&str, String> for UppercasePipe {
    fn transform(&self, value: &str) -> String {
        value.to_uppercase()
    }
}

impl Pipe<String, String> for UppercasePipe {
    fn transform(&self, value: String) -> String {
        value.to_uppercase()
    }
}

/// Lowercase pipe.
pub struct LowercasePipe;

impl Pipe<&str, String> for LowercasePipe {
    fn transform(&self, value: &str) -> String {
        value.to_lowercase()
    }
}

/// Currency pipe.
pub struct CurrencyPipe {
    pub symbol: &'static str,
    pub decimals: usize,
}

impl Default for CurrencyPipe {
    fn default() -> Self {
        Self {
            symbol: "$",
            decimals: 2,
        }
    }
}

impl Pipe<f64, String> for CurrencyPipe {
    fn transform(&self, value: f64) -> String {
        format!("{}{:.decimals$}", self.symbol, value, decimals = self.decimals)
    }
}

/// Date pipe (simplified).
pub struct DatePipe {
    pub format: &'static str,
}

impl Default for DatePipe {
    fn default() -> Self {
        Self { format: "%Y-%m-%d" }
    }
}

// ============================================================================
// Macros for Angular-like syntax
// ============================================================================

/// Define a component with Angular-like syntax.
///
/// # Example
///
/// ```rust,ignore
/// define_component! {
///     name: "my-counter",
///     state: { count: i32 = 0 },
///     template: |state, ctx| {
///         col![
///             label!(format!("Count: {}", state.count)),
///             button!("+", { ctx.update(|s| s.count += 1); }),
///         ]
///     },
///     styles: r#"
///         .counter { padding: 16px; }
///     "#,
///     on_init: |state| { println!("Init: {}", state.count); },
/// }
/// ```
#[macro_export]
macro_rules! define_component {
    (
        name: $name:expr,
        state: { $($field:ident : $type:ty = $default:expr),* $(,)? },
        template: $template:expr
        $(, styles: $styles:expr)?
        $(, on_init: $on_init:expr)?
        $(, on_destroy: $on_destroy:expr)?
        $(,)?
    ) => {{
        #[derive(Clone, Default)]
        struct State {
            $($field: $type,)*
        }

        let initial_state = State {
            $($field: $default,)*
        };

        let mut builder = $crate::component::component::<State, _>($name)
            .state(initial_state)
            .template($template);

        $(
            builder = builder.styles($styles);
        )?

        $(
            builder = builder.on_init($on_init);
        )?

        $(
            builder = builder.on_destroy($on_destroy);
        )?

        builder.build()
    }};
}

/// Structural if directive.
///
/// # Example
///
/// ```rust,ignore
/// ng_if!(show_content,
///     then: label!("Content is visible"),
///     else: label!("Content is hidden"),
/// )
/// ```
#[macro_export]
macro_rules! ng_if {
    ($cond:expr, then: $then:expr $(, else: $else:expr)? $(,)?) => {{
        let directive = $crate::component::If::new($cond).then($then);
        $(
            let directive = directive.otherwise($else);
        )?
        directive.render()
    }};
}

/// Structural for directive.
///
/// # Example
///
/// ```rust,ignore
/// ng_for!(items, |item, index| {
///     label!(format!("{}: {}", index, item))
/// })
/// ```
#[macro_export]
macro_rules! ng_for {
    ($items:expr, |$item:ident| $template:expr) => {{
        $crate::component::For::each($items, |$item, _| $template).render()
    }};

    ($items:expr, |$item:ident, $index:ident| $template:expr) => {{
        $crate::component::For::each($items, |$item, $index| $template).render()
    }};
}

/// Structural switch directive.
///
/// # Example
///
/// ```rust,ignore
/// ng_switch!(status,
///     "loading" => label!("Loading..."),
///     "error" => label!("Error!"),
///     "ready" => button!("Start"),
///     _ => label!("Unknown"),
/// )
/// ```
#[macro_export]
macro_rules! ng_switch {
    ($value:expr, $($case:pat => $widget:expr),+ $(, _ => $default:expr)? $(,)?) => {{
        match $value {
            $($case => Some($widget),)+
            $(_ => Some($default),)?
            #[allow(unreachable_patterns)]
            _ => None,
        }
    }};
}

/// Property binding macro (like [property]="value").
///
/// # Example
///
/// ```rust,ignore
/// let btn = bind!(Button::new("Click"),
///     disabled: !is_valid,
///     class: if active { "active" } else { "" },
/// );
/// ```
#[macro_export]
macro_rules! bind {
    ($widget:expr, $($prop:ident : $value:expr),* $(,)?) => {{
        let mut widget = $widget;
        $(
            widget = widget.$prop($value);
        )*
        widget
    }};
}

/// Event binding macro (like (event)="handler()").
///
/// # Example
///
/// ```rust,ignore
/// let btn = on!(Button::new("Click"),
///     click: || println!("Clicked!"),
///     hover: || println!("Hovered!"),
/// );
/// ```
#[macro_export]
macro_rules! on {
    ($widget:expr, click: $handler:expr $(,)?) => {{
        $widget.on_click($handler)
    }};

    ($widget:expr, change: $handler:expr $(,)?) => {{
        $widget.on_change($handler)
    }};

    ($widget:expr, submit: $handler:expr $(,)?) => {{
        $widget.on_submit($handler)
    }};
}

/// Two-way binding macro (like [(ngModel)]="value").
///
/// # Example
///
/// ```rust,ignore
/// let model = Model::new(String::new());
/// let input = model!(TextField::new(), model);
/// ```
#[macro_export]
macro_rules! model {
    ($widget:expr, $model:expr) => {{
        let model = $model.clone();
        $widget
            .value($model.value())
            .on_change(move |v| model.set(v.to_string()))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state() {
        let state = State::new(42);
        assert_eq!(*state.get(), 42);

        state.set(100);
        assert_eq!(*state.get(), 100);

        state.update(|v| *v += 1);
        assert_eq!(*state.get(), 101);
    }

    #[test]
    fn test_event_emitter() {
        let emitter = EventEmitter::<i32>::new();
        let received = Rc::new(RefCell::new(0));
        let received_clone = received.clone();

        emitter.subscribe(move |v| {
            *received_clone.borrow_mut() = v;
        });

        emitter.emit(42);
        assert_eq!(*received.borrow(), 42);
    }

    #[test]
    fn test_if_directive() {
        let result = If::new(true)
            .then("yes")
            .otherwise("no")
            .render();
        assert_eq!(result, Some("yes"));

        let result = If::new(false)
            .then("yes")
            .otherwise("no")
            .render();
        assert_eq!(result, Some("no"));
    }

    #[test]
    fn test_for_directive() {
        let items = vec!["a", "b", "c"];
        let result = For::each(items, |item, i| format!("{}: {}", i, item)).render();
        assert_eq!(result, vec!["0: a", "1: b", "2: c"]);
    }

    #[test]
    fn test_switch_directive() {
        let result = Switch::on("b")
            .case("a", 1)
            .case("b", 2)
            .case("c", 3)
            .default(0)
            .render();
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_binding() {
        let binding = Binding::new(10);
        assert_eq!(*binding.get(), 10);

        binding.set(20);
        assert_eq!(*binding.get(), 20);
    }
}
