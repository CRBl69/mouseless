use std::collections::{HashMap, HashSet};

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, PropType, UinputAbsSetup,
};
use log::{error, info};

use crate::messages::{ButtonEventType, MouseButton, TouchMessage};

#[derive(Clone)]
pub struct Touch {
    slot: i32,
    tracking_id: i32,
}

impl Touch {
    pub fn new(slot: i32, tracking_id: i32) -> Self {
        Self { slot, tracking_id }
    }
}

pub struct State {
    slot: i32,
    tool: Option<Key>,
    touch: bool,
    tracking_id: i32,
}

pub struct VirtualTrackpad {
    virtual_device: VirtualDevice,
    touches: HashMap<i32, Touch>,
    state: State,
}

impl VirtualTrackpad {
    pub fn new(x: i32, y: i32) -> Self {
        let x_mt_axis = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_MT_POSITION_X,
            AbsInfo::new(0, 0, x, 0, 0, 100),
        );
        let y_mt_axis = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_MT_POSITION_Y,
            AbsInfo::new(0, 0, y, 0, 0, 100),
        );
        let x_axis = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, AbsInfo::new(0, 0, x, 0, 0, 100));
        let y_axis = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, AbsInfo::new(0, 0, y, 0, 0, 100));
        let slots = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_MT_SLOT,
            AbsInfo::new(0, 0, 5, 0, 0, 0),
        );
        let tracks = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_MT_TRACKING_ID,
            AbsInfo::new(0, 0, 1000, 0, 0, 0),
        );
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::BTN_LEFT);
        keys.insert(Key::BTN_RIGHT);
        keys.insert(Key::BTN_TOOL_FINGER);
        keys.insert(Key::BTN_TOUCH);
        keys.insert(Key::KEY_SCROLLDOWN);
        keys.insert(Key::KEY_SCROLLUP);
        let mut trackpad_info = AttributeSet::<PropType>::new();
        trackpad_info.insert(PropType::POINTER);
        let device = VirtualDeviceBuilder::new()
            .unwrap()
            .name("mouseless_virtual_trackpad")
            .with_absolute_axis(&x_axis)
            .unwrap()
            .with_absolute_axis(&y_axis)
            .unwrap()
            .with_absolute_axis(&x_mt_axis)
            .unwrap()
            .with_absolute_axis(&y_mt_axis)
            .unwrap()
            .with_absolute_axis(&slots)
            .unwrap()
            .with_absolute_axis(&tracks)
            .unwrap()
            .with_keys(&keys)
            .unwrap()
            .with_properties(&trackpad_info)
            .unwrap()
            .build();
        match device {
            Ok(device) => VirtualTrackpad {
                virtual_device: device,
                touches: HashMap::new(),
                state: State {
                    slot: -1,
                    tool: None,
                    touch: false,
                    tracking_id: 0,
                },
            },
            Err(e) => {
                error!("Could not initialize VirtualTrackpad: {:#?}", e);
                panic!();
            }
        }
    }

    pub fn click(&mut self, button: MouseButton, event_type: ButtonEventType) {
        let click_event = InputEvent::new(
            EventType::KEY,
            if matches!(button, MouseButton::Left) {
                Key::BTN_LEFT.0
            } else {
                Key::BTN_RIGHT.0
            },
            if matches!(event_type, ButtonEventType::Down) {
                1
            } else {
                0
            },
        );
        self.virtual_device.emit(&[click_event]).unwrap();
    }

    fn touch_number_events(&mut self) -> Vec<InputEvent> {
        let mut events = vec![];
        if let Some(tool) = self.state.tool {
            events.push(InputEvent::new_now(EventType::KEY, tool.0, 0));
        }
        if self.touches.is_empty() && self.state.touch {
            events.push(InputEvent::new_now(EventType::KEY, Key::BTN_TOUCH.0, 0));
            self.state.touch = false;
        }
        if !self.touches.is_empty() && !self.state.touch {
            events.push(InputEvent::new_now(EventType::KEY, Key::BTN_TOUCH.0, 1));
            self.state.touch = true;
        }
        let tool = match self.touches.len() {
            1 => Key::BTN_TOOL_FINGER,
            2 => Key::BTN_TOOL_DOUBLETAP,
            3 => Key::BTN_TOOL_TRIPLETAP,
            _ => Key::BTN_TOOL_QUADTAP,
        };
        events.push(InputEvent::new_now(EventType::KEY, tool.0, 1));
        self.state.tool = Some(tool);
        events
    }

    fn xy_events(&mut self, x: i32, y: i32) -> Vec<InputEvent> {
        vec![
            InputEvent::new_now(
                EventType::ABSOLUTE,
                AbsoluteAxisType::ABS_MT_POSITION_X.0,
                x,
            ),
            InputEvent::new_now(
                EventType::ABSOLUTE,
                AbsoluteAxisType::ABS_MT_POSITION_Y.0,
                y,
            ),
            InputEvent::new_now(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, x),
            InputEvent::new_now(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, y),
        ]
    }

    fn switch_slots(&mut self, slot: i32) -> Vec<InputEvent> {
        if slot != self.state.slot {
            self.state.slot = slot;
            vec![InputEvent::new_now(
                EventType::ABSOLUTE,
                AbsoluteAxisType::ABS_MT_SLOT.0,
                slot,
            )]
        } else {
            vec![]
        }
    }

    fn switch_tracking_id(&mut self, tracking_id: i32) -> Vec<InputEvent> {
        if tracking_id != self.state.tracking_id {
            self.state.tracking_id = tracking_id;
            vec![InputEvent::new_now(
                EventType::ABSOLUTE,
                AbsoluteAxisType::ABS_MT_TRACKING_ID.0,
                tracking_id,
            )]
        } else {
            vec![]
        }
    }

    fn get_first_free_slot(&mut self) -> i32 {
        let mut slot = 0;
        let slots = self
            .touches
            .values()
            .map(|v| v.slot)
            .collect::<HashSet<i32>>();
        while slots.contains(&slot) {
            slot += 1;
        }
        slot
    }

    pub fn trackpad(&mut self, touches: TouchMessage) {
        let mut events = vec![];

        for touch in touches.changed_touches {
            if let Some(t) = self.touches.get(&touch.id).cloned() {
                events.extend(self.switch_slots(t.slot));
                events.extend(self.switch_tracking_id(t.tracking_id));
            } else {
                self.state.slot = self.get_first_free_slot();
                self.state.tracking_id += 1;
                self.touches.insert(
                    touch.id,
                    Touch::new(self.state.slot, self.state.tracking_id),
                );
                events.extend(self.switch_slots(self.state.slot));
                events.extend(self.switch_tracking_id(self.state.tracking_id));
                events.extend(self.touch_number_events());
            }

            events.extend(self.xy_events(touch.x, touch.y));
        }

        for id in touches.released_touches {
            if let Some(t) = self.touches.get(&id) {
                events.extend(self.switch_slots(t.slot));
                events.push(InputEvent::new_now(
                    EventType::ABSOLUTE,
                    AbsoluteAxisType::ABS_MT_TRACKING_ID.0,
                    -1,
                ));
                self.touches.remove(&id);
                events.extend(self.touch_number_events());
            }
        }
        info!("events: {:#?}", events);
        self.virtual_device.emit(&events).unwrap();
    }
}
