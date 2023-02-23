use serde::Deserialize;

#[derive(Deserialize)]
pub enum Channel {
    Trackpad(TrackpadMessage),
    Gamepad,
    Keyboard,
}

#[derive(Deserialize)]
pub enum TrackpadMessage {
    TouchMessage(TouchMessage),
    ClickMessage(ClickMessage),
    InitMessage(InitMessage),
}

#[derive(Deserialize)]
pub struct TouchMessage {
    pub changed_touches: Vec<Touch>,
    pub released_touches: Vec<i32>,
}

#[derive(Deserialize)]
pub struct Touch {
    pub x: i32,
    pub y: i32,
    pub id: i32,
}

#[derive(Deserialize)]
pub struct ClickMessage {
    pub button: MouseButton,
    pub event_type: ButtonEventType,
}

#[derive(Deserialize)]
pub struct InitMessage {
    pub x: i32,
    pub y: i32,
}

#[derive(Deserialize)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(Deserialize)]
pub enum ButtonEventType {
    Down,
    Up,
}
