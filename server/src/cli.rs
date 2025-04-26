use clap::Parser;
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    #[arg(short, long, default_value_t = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 8080, 0, 0)))]
    pub listen_address: SocketAddr,

    #[arg(short, long)]
    pub metrics_listen_address: Option<SocketAddr>,

    #[arg(short = 'a', long)]
    pub host_url: Url,

    #[arg(short, long)]
    pub database_url: String,

    #[arg(short, long)]
    pub openid_provider_url: Url,

    #[arg(short = 'c', long)]
    pub openid_client_id: String,

    #[arg(short = 's', long)]
    pub openid_client_secret: String,
}
