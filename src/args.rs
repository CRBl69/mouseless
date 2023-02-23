use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The port to listen on
    #[arg(short, long, default_value_t = 3030)]
    pub port: u16,

    /// The IP address to bind to
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    pub ip: IpAddr,
}
