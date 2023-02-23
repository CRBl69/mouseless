use futures_util::StreamExt;
use log::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use warp::ws;

use crate::{
    drivers,
    messages::{self, TrackpadMessage},
};

pub struct Drivers {
    trackpad_driver: Option<Arc<Mutex<drivers::VirtualTrackpad>>>,
}

pub async fn ws_upgrade(websocket: ws::WebSocket) {
    info!("New websocket connection");
    let (_tx, mut rx) = websocket.split();
    let mut drivers = Drivers {
        trackpad_driver: None,
    };
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            match message {
                Err(e) => {
                    error!("Websocket error: {e:?}");
                }
                Ok(message) => {
                    if message.is_text() {
                        let json = message.to_str().unwrap();
                        debug!("Data received: {json}");
                        let data: messages::Channel = serde_json::from_str(json).unwrap();
                        handle_message(data, &mut drivers).await;
                    }
                }
            }
        }
    });
}

pub async fn handle_message(message: messages::Channel, drivers: &mut Drivers) {
    match message {
        messages::Channel::Trackpad(trackpad_msg) => match trackpad_msg {
            TrackpadMessage::InitMessage(init_data) => {
                drivers.trackpad_driver = Some(Arc::new(Mutex::new(
                    drivers::VirtualTrackpad::new(init_data.x, init_data.y),
                )));
                info!("Trackpad driver initialized");
            }
            _ => {
                if let Some(driver) = &drivers.trackpad_driver {
                    let mut driver = driver.lock().unwrap();
                    match trackpad_msg {
                        TrackpadMessage::ClickMessage(data) => {
                            driver.click(data.button, data.event_type);
                        }
                        TrackpadMessage::TouchMessage(data) => {
                            driver.trackpad(data);
                        }
                        TrackpadMessage::InitMessage(_) => {
                            warn!("InitMessage was sent after initialization, ignoring");
                        }
                    }
                } else {
                    error!("Trackpad driver not initialized");
                }
            }
        },
        _ => {
            error!("Message type unimplemented");
        }
    };
}
