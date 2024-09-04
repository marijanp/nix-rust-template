use clap::Parser;
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    #[arg(short, long, default_value_t = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 8080, 0, 0)))]
    pub listen_address: SocketAddr,
}
