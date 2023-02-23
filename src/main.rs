use args::Args;
use clap::Parser;
use warp::{ws, Filter};

mod args;
mod drivers;
mod messages;
mod websocket;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let hello = warp::path::end()
        .and(warp::filters::ws::ws())
        .map(|ws: ws::Ws| ws.on_upgrade(websocket::ws_upgrade));

    warp::serve(hello).run((args.ip, args.port)).await;
}
