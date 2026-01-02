//! AI Chat Window widget
//!
//! A modern chat interface for interacting with AI assistants.
//! Supports multiple AI providers including local models and cloud APIs.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, Key, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A single message in the chat
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Unique message ID
    pub id: String,
    /// Message role (user, assistant, system)
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: String,
    /// Whether the message is still streaming
    pub is_streaming: bool,
    /// Token count (if available)
    pub tokens: Option<u32>,
}

impl ChatMessage {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: uuid_simple(),
            role: MessageRole::User,
            content: content.into(),
            timestamp: current_time(),
            is_streaming: false,
            tokens: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: uuid_simple(),
            role: MessageRole::Assistant,
            content: content.into(),
            timestamp: current_time(),
            is_streaming: false,
            tokens: None,
        }
    }

    /// Create a streaming assistant message
    pub fn assistant_streaming() -> Self {
        Self {
            id: uuid_simple(),
            role: MessageRole::Assistant,
            content: String::new(),
            timestamp: current_time(),
            is_streaming: true,
            tokens: None,
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: uuid_simple(),
            role: MessageRole::System,
            content: content.into(),
            timestamp: current_time(),
            is_streaming: false,
            tokens: None,
        }
    }
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// AI Provider type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiProvider {
    /// Local Ollama instance
    Ollama { host: String, model: String },
    /// Local LM Studio
    LmStudio { host: String, model: String },
    /// Local llama.cpp server
    LlamaCpp { host: String },
    /// OpenAI API (BYOK)
    OpenAi { api_key: String, model: String },
    /// Anthropic Claude API (BYOK)
    Claude { api_key: String, model: String },
    /// Custom OpenAI-compatible endpoint
    OpenAiCompatible { host: String, api_key: Option<String>, model: String },
}

impl AiProvider {
    /// Create Ollama provider with default settings
    pub fn ollama(model: impl Into<String>) -> Self {
        Self::Ollama {
            host: "http://localhost:11434".to_string(),
            model: model.into(),
        }
    }

    /// Create LM Studio provider
    pub fn lm_studio(model: impl Into<String>) -> Self {
        Self::LmStudio {
            host: "http://localhost:1234".to_string(),
            model: model.into(),
        }
    }

    /// Create OpenAI provider
    pub fn openai(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::OpenAi {
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    /// Create Claude provider
    pub fn claude(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::Claude {
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    /// Get provider display name
    pub fn display_name(&self) -> &str {
        match self {
            AiProvider::Ollama { .. } => "Ollama",
            AiProvider::LmStudio { .. } => "LM Studio",
            AiProvider::LlamaCpp { .. } => "llama.cpp",
            AiProvider::OpenAi { .. } => "OpenAI",
            AiProvider::Claude { .. } => "Claude",
            AiProvider::OpenAiCompatible { .. } => "Custom API",
        }
    }

    /// Get model name
    pub fn model_name(&self) -> &str {
        match self {
            AiProvider::Ollama { model, .. } => model,
            AiProvider::LmStudio { model, .. } => model,
            AiProvider::LlamaCpp { .. } => "default",
            AiProvider::OpenAi { model, .. } => model,
            AiProvider::Claude { model, .. } => model,
            AiProvider::OpenAiCompatible { model, .. } => model,
        }
    }
}

/// Chat window configuration
#[derive(Debug, Clone)]
pub struct ChatConfig {
    /// System prompt
    pub system_prompt: String,
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,
    /// Max tokens to generate
    pub max_tokens: u32,
    /// Enable streaming responses
    pub streaming: bool,
    /// Show token counts
    pub show_tokens: bool,
    /// Show timestamps
    pub show_timestamps: bool,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            system_prompt: "You are a helpful AI assistant.".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            streaming: true,
            show_tokens: false,
            show_timestamps: true,
        }
    }
}

/// Chat window state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChatState {
    #[default]
    Idle,
    Connecting,
    Streaming,
    Error,
}

/// AI Chat Window widget
pub struct ChatWindow {
    base: WidgetBase,

    // Provider
    provider: Option<AiProvider>,
    config: ChatConfig,

    // Messages
    messages: Vec<ChatMessage>,

    // Input
    input_text: String,
    input_focused: bool,
    cursor_position: usize,

    // Scroll
    scroll_offset: f32,
    auto_scroll: bool,

    // State
    state: ChatState,
    error_message: Option<String>,

    // Sizing
    width: f32,
    height: f32,
    input_height: f32,
    message_padding: f32,

    // Colors
    user_bubble_color: Color,
    assistant_bubble_color: Color,
    system_bubble_color: Color,
    input_bg_color: Color,
    text_color: Color,
    placeholder_color: Color,
    accent_color: Color,

    // Callbacks
    #[allow(clippy::type_complexity)]
    on_send: Option<Box<dyn Fn(&str, &[ChatMessage]) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_provider_change: Option<Box<dyn Fn(&AiProvider) + Send + Sync>>,
}

impl ChatWindow {
    /// Create a new chat window
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("chat-window"),
            provider: None,
            config: ChatConfig::default(),
            messages: Vec::new(),
            input_text: String::new(),
            input_focused: false,
            cursor_position: 0,
            scroll_offset: 0.0,
            auto_scroll: true,
            state: ChatState::Idle,
            error_message: None,
            width: 400.0,
            height: 600.0,
            input_height: 48.0,
            message_padding: 12.0,
            user_bubble_color: Color::rgb(0.0, 0.47, 0.84),
            assistant_bubble_color: Color::rgb(0.2, 0.2, 0.25),
            system_bubble_color: Color::rgb(0.3, 0.25, 0.2),
            input_bg_color: Color::rgb(0.15, 0.15, 0.18),
            text_color: Color::WHITE,
            placeholder_color: Color::rgba(1.0, 1.0, 1.0, 0.5),
            accent_color: Color::rgb(0.0, 0.47, 0.84),
            on_send: None,
            on_provider_change: None,
        }
    }

    /// Set the AI provider
    pub fn provider(mut self, provider: AiProvider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the chat configuration
    pub fn config(mut self, config: ChatConfig) -> Self {
        self.config = config;
        self
    }

    /// Set system prompt
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = prompt.into();
        self
    }

    /// Set window size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set user bubble color
    pub fn user_color(mut self, color: Color) -> Self {
        self.user_bubble_color = color;
        self
    }

    /// Set assistant bubble color
    pub fn assistant_color(mut self, color: Color) -> Self {
        self.assistant_bubble_color = color;
        self
    }

    /// Set accent color
    pub fn accent_color(mut self, color: Color) -> Self {
        self.accent_color = color;
        self
    }

    /// Set send callback
    pub fn on_send<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[ChatMessage]) + Send + Sync + 'static,
    {
        self.on_send = Some(Box::new(f));
        self
    }

    /// Add a message
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Update the last assistant message (for streaming)
    pub fn update_streaming(&mut self, content: &str) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == MessageRole::Assistant && last.is_streaming {
                last.content.push_str(content);
            }
        }
    }

    /// Complete streaming
    pub fn complete_streaming(&mut self) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == MessageRole::Assistant {
                last.is_streaming = false;
            }
        }
        self.state = ChatState::Idle;
    }

    /// Set error state
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.state = ChatState::Error;
        self.error_message = Some(message.into());
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.state = ChatState::Idle;
        self.error_message = None;
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0.0;
    }

    /// Get messages
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Get current provider
    pub fn get_provider(&self) -> Option<&AiProvider> {
        self.provider.as_ref()
    }

    /// Set provider at runtime
    pub fn set_provider(&mut self, provider: AiProvider) {
        self.provider = Some(provider.clone());
        if let Some(ref cb) = self.on_provider_change {
            cb(&provider);
        }
    }

    /// Scroll to bottom
    fn scroll_to_bottom(&mut self) {
        // Will be calculated during layout
        self.scroll_offset = f32::MAX;
    }

    /// Send current input
    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }

        let content = std::mem::take(&mut self.input_text);
        self.cursor_position = 0;

        // Add user message
        self.add_message(ChatMessage::user(&content));

        // Start streaming response
        self.add_message(ChatMessage::assistant_streaming());
        self.state = ChatState::Streaming;

        // Trigger callback
        if let Some(ref cb) = self.on_send {
            cb(&content, &self.messages);
        }
    }

    /// Add CSS class
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Set element ID
    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }

    /// Calculate message height
    fn message_height(&self, message: &ChatMessage) -> f32 {
        let chars_per_line = ((self.width - self.message_padding * 4.0) / 8.0) as usize;
        let lines = (message.content.len() / chars_per_line.max(1) + 1) as f32;
        (lines * 20.0 + self.message_padding * 2.0).max(48.0)
    }

    /// Draw a message bubble
    fn draw_message(&self, painter: &mut Painter, message: &ChatMessage, rect: Rect) {
        let (bg_color, align_right) = match message.role {
            MessageRole::User => (self.user_bubble_color, true),
            MessageRole::Assistant => (self.assistant_bubble_color, false),
            MessageRole::System => (self.system_bubble_color, false),
        };

        let bubble_width = (rect.width() * 0.8).min(rect.width() - 32.0);
        let bubble_x = if align_right {
            rect.max_x() - bubble_width - 16.0
        } else {
            rect.x() + 16.0
        };

        let bubble_rect = Rect::new(bubble_x, rect.y(), bubble_width, rect.height() - 8.0);
        painter.fill_rounded_rect(bubble_rect, bg_color, BorderRadius::all(12.0));

        // Message content
        let text_x = bubble_rect.x() + self.message_padding;
        let text_y = bubble_rect.y() + self.message_padding + 14.0;

        // Simple text wrapping (draw first portion)
        let display_content = if message.content.len() > 500 {
            format!("{}...", &message.content[..500])
        } else {
            message.content.clone()
        };

        painter.draw_text(
            &display_content,
            Point::new(text_x, text_y),
            self.text_color,
            14.0,
        );

        // Streaming indicator
        if message.is_streaming {
            let cursor = "▊";
            painter.draw_text(
                cursor,
                Point::new(text_x + (message.content.len() as f32 * 8.0).min(bubble_width - 24.0), text_y),
                self.accent_color,
                14.0,
            );
        }

        // Timestamp
        if self.config.show_timestamps && !message.timestamp.is_empty() {
            painter.draw_text(
                &message.timestamp,
                Point::new(bubble_rect.x() + self.message_padding, bubble_rect.max_y() - 6.0),
                self.placeholder_color,
                10.0,
            );
        }
    }
}

impl Default for ChatWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ChatWindow {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "chat-window"
    }

    fn element_id(&self) -> Option<&str> {
        self.base.element_id.as_deref()
    }

    fn classes(&self) -> &ClassList {
        &self.base.classes
    }

    fn state(&self) -> WidgetState {
        self.base.state
    }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        // Background
        painter.fill_rounded_rect(rect, Color::rgb(0.1, 0.1, 0.12), BorderRadius::all(12.0));

        // Header
        let _header_rect = Rect::new(rect.x(), rect.y(), rect.width(), 48.0);
        painter.fill_rounded_rect(
            Rect::new(rect.x(), rect.y(), rect.width(), 48.0),
            Color::rgb(0.12, 0.12, 0.15),
            BorderRadius::new(12.0, 12.0, 0.0, 0.0),
        );

        // Title
        let title = if let Some(ref provider) = self.provider {
            format!("AI Chat - {} ({})", provider.display_name(), provider.model_name())
        } else {
            "AI Chat - No Provider".to_string()
        };
        painter.draw_text(
            &title,
            Point::new(rect.x() + 16.0, rect.y() + 30.0),
            self.text_color,
            14.0,
        );

        // Status indicator
        let status_color = match self.state {
            ChatState::Idle => Color::rgb(0.2, 0.8, 0.2),
            ChatState::Connecting => Color::rgb(0.8, 0.8, 0.2),
            ChatState::Streaming => Color::rgb(0.2, 0.6, 1.0),
            ChatState::Error => Color::rgb(0.8, 0.2, 0.2),
        };
        let status_rect = Rect::new(rect.max_x() - 24.0, rect.y() + 20.0, 8.0, 8.0);
        painter.fill_rounded_rect(status_rect, status_color, BorderRadius::all(4.0));

        // Messages area
        let messages_rect = Rect::new(
            rect.x(),
            rect.y() + 48.0,
            rect.width(),
            rect.height() - 48.0 - self.input_height - 16.0,
        );

        // Draw messages
        let mut y = messages_rect.y() + 8.0 - self.scroll_offset;
        for message in &self.messages {
            let msg_height = self.message_height(message);
            if y + msg_height > messages_rect.y() && y < messages_rect.max_y() {
                let msg_rect = Rect::new(messages_rect.x(), y, messages_rect.width(), msg_height);
                self.draw_message(painter, message, msg_rect);
            }
            y += msg_height + 8.0;
        }

        // Empty state
        if self.messages.is_empty() {
            painter.draw_text(
                "Start a conversation...",
                Point::new(messages_rect.x() + 16.0, messages_rect.y() + 40.0),
                self.placeholder_color,
                14.0,
            );

            if self.provider.is_none() {
                painter.draw_text(
                    "No AI provider configured",
                    Point::new(messages_rect.x() + 16.0, messages_rect.y() + 70.0),
                    Color::rgb(0.8, 0.5, 0.2),
                    12.0,
                );
            }
        }

        // Error message
        if let Some(ref error) = self.error_message {
            let error_rect = Rect::new(
                rect.x() + 16.0,
                messages_rect.max_y() - 40.0,
                rect.width() - 32.0,
                32.0,
            );
            painter.fill_rounded_rect(error_rect, Color::rgba(0.8, 0.2, 0.2, 0.9), BorderRadius::all(6.0));
            painter.draw_text(
                error,
                Point::new(error_rect.x() + 8.0, error_rect.y() + 20.0),
                Color::WHITE,
                12.0,
            );
        }

        // Input area
        let input_rect = Rect::new(
            rect.x() + 12.0,
            rect.max_y() - self.input_height - 8.0,
            rect.width() - 24.0,
            self.input_height,
        );

        // Input background
        let input_border_color = if self.input_focused {
            self.accent_color
        } else {
            Color::rgba(1.0, 1.0, 1.0, 0.1)
        };
        painter.fill_rounded_rect(input_rect, self.input_bg_color, BorderRadius::all(24.0));

        // Input border when focused
        if self.input_focused {
            let border_rect = Rect::new(
                input_rect.x() - 1.0,
                input_rect.y() - 1.0,
                input_rect.width() + 2.0,
                input_rect.height() + 2.0,
            );
            painter.fill_rounded_rect(border_rect, input_border_color, BorderRadius::all(25.0));
            painter.fill_rounded_rect(input_rect, self.input_bg_color, BorderRadius::all(24.0));
        }

        // Input text or placeholder
        let text_to_draw = if self.input_text.is_empty() {
            ("Type a message...", self.placeholder_color)
        } else {
            (self.input_text.as_str(), self.text_color)
        };
        painter.draw_text(
            text_to_draw.0,
            Point::new(input_rect.x() + 16.0, input_rect.y() + 30.0),
            text_to_draw.1,
            14.0,
        );

        // Cursor
        if self.input_focused && !self.input_text.is_empty() {
            let cursor_x = input_rect.x() + 16.0 + (self.cursor_position as f32 * 8.0);
            let cursor_rect = Rect::new(cursor_x, input_rect.y() + 12.0, 2.0, 24.0);
            painter.fill_rect(cursor_rect, self.accent_color);
        }

        // Send button
        let send_btn_rect = Rect::new(
            input_rect.max_x() - 40.0,
            input_rect.y() + 8.0,
            32.0,
            32.0,
        );
        let send_enabled = !self.input_text.trim().is_empty() && self.state == ChatState::Idle;
        let send_color = if send_enabled {
            self.accent_color
        } else {
            Color::rgba(1.0, 1.0, 1.0, 0.3)
        };
        painter.fill_rounded_rect(send_btn_rect, send_color, BorderRadius::all(16.0));

        // Send arrow icon (simple triangle)
        painter.draw_text(
            "→",
            Point::new(send_btn_rect.x() + 8.0, send_btn_rect.y() + 22.0),
            Color::WHITE,
            16.0,
        );
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(MouseEvent { kind, position, button, .. }) => {
                let in_bounds = self.base.bounds.contains(*position);

                // Calculate input rect
                let input_rect = Rect::new(
                    self.base.bounds.x() + 12.0,
                    self.base.bounds.max_y() - self.input_height - 8.0,
                    self.base.bounds.width() - 24.0,
                    self.input_height,
                );
                let in_input = input_rect.contains(*position);

                // Send button
                let send_btn_rect = Rect::new(
                    input_rect.max_x() - 40.0,
                    input_rect.y() + 8.0,
                    32.0,
                    32.0,
                );
                let in_send = send_btn_rect.contains(*position);

                match kind {
                    MouseEventKind::Down if *button == Some(MouseButton::Left) => {
                        if in_send && self.state == ChatState::Idle {
                            self.send_message();
                            return EventResult::Handled;
                        }
                        if in_input {
                            self.input_focused = true;
                            return EventResult::Handled;
                        }
                        if in_bounds && !in_input {
                            self.input_focused = false;
                        }
                    }
                    MouseEventKind::Scroll { delta_y, .. } if in_bounds => {
                        self.scroll_offset = (self.scroll_offset - (*delta_y as f32) * 20.0).max(0.0);
                        self.auto_scroll = false;
                        return EventResult::Handled;
                    }
                    _ => {}
                }
            }
            Event::Key(KeyEvent { kind: KeyEventKind::Down, key, text, .. }) => {
                if self.input_focused {
                    match key {
                        Key::Enter => {
                            if self.state == ChatState::Idle {
                                self.send_message();
                            }
                            return EventResult::Handled;
                        }
                        Key::Backspace => {
                            if self.cursor_position > 0 {
                                self.cursor_position -= 1;
                                self.input_text.remove(self.cursor_position);
                            }
                            return EventResult::Handled;
                        }
                        Key::Left => {
                            if self.cursor_position > 0 {
                                self.cursor_position -= 1;
                            }
                            return EventResult::Handled;
                        }
                        Key::Right => {
                            if self.cursor_position < self.input_text.len() {
                                self.cursor_position += 1;
                            }
                            return EventResult::Handled;
                        }
                        Key::Home => {
                            self.cursor_position = 0;
                            return EventResult::Handled;
                        }
                        Key::End => {
                            self.cursor_position = self.input_text.len();
                            return EventResult::Handled;
                        }
                        Key::Escape => {
                            self.input_focused = false;
                            return EventResult::Handled;
                        }
                        _ => {
                            // Handle text input
                            if let Some(t) = text {
                                for c in t.chars() {
                                    if !c.is_control() {
                                        self.input_text.insert(self.cursor_position, c);
                                        self.cursor_position += 1;
                                    }
                                }
                                return EventResult::Handled;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        EventResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;
    }
}

// Simple UUID generator
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", nanos)
}

// Get current time as string
fn current_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    format!("{:02}:{:02}", hours, mins)
}
