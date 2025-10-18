use bevy::input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseWheel},
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<InputEvent>()
            .add_systems(PreUpdate, (mouse_input, keyboard_input));
    }
}

/// Relative screen position, normalized at (0.0, 0.0) in the top-left,
/// with the each unit corresponding to a logical pixel.
#[derive(Debug, Deref, Clone, Copy)]
pub struct ScreenPosition(Vec2);
impl ScreenPosition {
    pub fn new(value: Vec2) -> Self {
        value.into()
    }
}
impl From<Vec2> for ScreenPosition {
    fn from(value: Vec2) -> Self {
        assert!(value.x >= 0.0, "Invalid ScreenPosition: {value}");
        assert!(value.y >= 0.0, "Invalid ScreenPosition: {value}");
        Self(value)
    }
}

#[derive(Debug, Message)]
pub enum InputEvent {
    /// Clear a block at a screen position (default: left click).
    ClearBlock(ScreenPosition),
    /// Mark a block at a screen location (default: Right click).
    MarkBlock(ScreenPosition),
    /// Select a button or object at a position on the screen (default: left click).
    Select(ScreenPosition),
    /// Rotate the camera. `delta.x` is horizontal rotation, and `delta.y` is vertical
    /// (default: Middle mouse button and movement).
    /// Relative to window size.
    RotateCamera { delta: Vec2 },
    /// Zoom the camera (default mouse wheel up/down).
    /// `delta` indicates zoom direction and magnitude: positive zooms in, and negative zooms out.
    ZoomCamera { delta: f32 },
    /// Pause the game is a specific key is pressed (default ESC) or if the window
    /// (or app) loses focus.
    Pause,
}

/// Conversion factor between scroll by pixels and scroll by lines, for consistent
/// behavior on different devices.
const SCROLL_PIXELS_PER_LINE: f32 = 8.0;

/// Handle mouse input. All available events are consumed and accumulated into possibly fewer
/// InputEvents for efficiency.
fn mouse_input(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut mouse_button_events: MessageReader<MouseButtonInput>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut input_events: MessageWriter<InputEvent>,
) {
    // Get the singular primary window. Multiple windows is not handled.
    let window = primary_window.single().unwrap();
    // Handle mouse motion events only if the rotate button (default MMB) is pressed
    if mouse_button.pressed(MouseButton::Middle) {
        // Collect all motion events into a single delta
        let mut delta = Vec2::ZERO;
        for motion_event in mouse_motion_events.read() {
            delta += motion_event.delta;
        }
        // Only send a InputEvent::RotateCamera if the delta is nonzero
        if delta.length_squared() > 0.0 {
            // Scale based on window size
            let delta = Vec2::new(delta.x / window.width(), delta.y / window.height());
            log::debug!("Send InputEvent::RotateCamera");
            input_events.write(InputEvent::RotateCamera { delta });
        }
    } else {
        // If the rotate button is not pressed, clear all rotation events
        mouse_motion_events.clear();
    }
    // Handle scroll events
    let mut scroll_delta = 0.0;
    // Collect all scroll events into a single delta
    for mouse_wheel_event in mouse_wheel_events.read() {
        let scroll = mouse_wheel_event.y;
        scroll_delta += match mouse_wheel_event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                log::debug!("Scrolled {scroll} lines");
                scroll
            }
            bevy::input::mouse::MouseScrollUnit::Pixel => {
                let lines = scroll * SCROLL_PIXELS_PER_LINE;
                log::debug!("Scrolled {scroll} pixels ({lines} lines)");
                lines
            }
        };
    }
    // Only send an event if the delta is nonzero
    if scroll_delta.abs() > 0.0 {
        log::debug!("Send InputEvent::ZoomCamera");
        input_events.write(InputEvent::ZoomCamera {
            delta: scroll_delta,
        });
    }
    // We don't care about mouse clicks if the mouse is not in the primary window
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    // Handle mouse click events (default LMB or RMB)
    for mouse_button_event in mouse_button_events.read() {
        if mouse_button_event.state.is_pressed() {
            log::debug!("Click at {cursor_pos:?}");
            match mouse_button_event.button {
                MouseButton::Left => {
                    log::debug!("Send InputEvent::ClearBlock");
                    input_events.write(InputEvent::ClearBlock(cursor_pos.into()));
                }
                MouseButton::Right => {
                    log::debug!("Send InputEvent::MarkBlock");
                    input_events.write(InputEvent::MarkBlock(cursor_pos.into()));
                }
                _ => {}
            };
        }
    }
}

fn keyboard_input(
    mut key_events: MessageReader<KeyboardInput>,
    mut input_events: MessageWriter<InputEvent>,
) {
    for key_event in key_events.read() {
        match key_event {
            KeyboardInput {
                key_code, state, ..
            } if matches!(key_code, KeyCode::Escape) && state.is_pressed() => {
                log::debug!("Send InputEvent::Pause");
                input_events.write(InputEvent::Pause);
            }
            _ => {}
        }
    }
}
