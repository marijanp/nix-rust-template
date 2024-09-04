use server::{cli::CliArgs, routes::GetItemsPath};

use backoff::{future::retry, ExponentialBackoff};
use std::net::SocketAddr;
use tokio::io;
use tokio::net::TcpStream;

async fn wait_for_port(address: &SocketAddr) -> Result<(), io::Error> {
    retry(ExponentialBackoff::default(), || async {
        Ok(TcpStream::connect(address).await.map(|_| ())?)
    })
    .await
}

#[tokio::test]
pub async fn test_server_listens() {
    let server_address = "0.0.0.0:8080";
    let _handler = tokio::spawn(async {
        server::run(CliArgs {
            listen_address: server_address.parse().unwrap(),
        })
        .await
    });
    wait_for_port(&server_address.parse().unwrap())
        .await
        .unwrap();
    tokio::task::spawn_blocking(move || {
        let response = ureq::get(&format!("http://{server_address}{GetItemsPath}"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(response, "Hello World!")
    })
    .await
    .unwrap();
}
