use server::{cli::CliArgs, routes::GetItemsPath};

use backoff::{future::retry, ExponentialBackoff};
use std::net::{SocketAddr, TcpListener};
use tokio::io;
use tokio::net::TcpStream;

async fn wait_for_port(address: &SocketAddr) -> Result<(), io::Error> {
    retry(ExponentialBackoff::default(), || async {
        Ok(TcpStream::connect(address)
            .await
            .map(|_| {
                tracing::info!("Port at {address} open!");
            })
            .map_err(|err| {
                tracing::error!("{address}: {err:?}");
                err
            })?)
    })
    .await
}

pub struct TestServer {
    task_handle: tokio::task::JoinHandle<io::Result<()>>,
    pub server_url: String,
    pub metrics_server_url: Option<String>,
}

impl TestServer {
    pub async fn run(with_metrics_server: bool) -> io::Result<TestServer> {
        let listen_address = TcpListener::bind("0.0.0.0:0")
            .unwrap()
            .local_addr()
            .unwrap();
        let metrics_listen_address = with_metrics_server.then_some(
            TcpListener::bind("0.0.0.0:0")
                .unwrap()
                .local_addr()
                .unwrap(),
        );
        let task_handle = tokio::spawn(async move {
            server::run(CliArgs {
                listen_address,
                metrics_listen_address,
            })
            .await
        });

        wait_for_port(&listen_address).await.unwrap();

        let metrics_server_url = match metrics_listen_address {
            Some(metrics_listen_address) => {
                wait_for_port(&metrics_listen_address).await.unwrap();
                Some(format!("http://{}", metrics_listen_address))
            }
            None => None,
        };

        Ok(TestServer {
            task_handle,
            server_url: format!("http://{}", listen_address),
            metrics_server_url,
        })
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task_handle.abort();
    }
}

#[tokio::test]
pub async fn test_server_listens() {
    let server = TestServer::run(false).await.unwrap();
    tokio::task::spawn_blocking(move || {
        let server_url = &server.server_url;
        let response = ureq::get(&format!("{server_url}{GetItemsPath}"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert!(response.contains("<table>"));
        assert!(response.contains("test"));
    })
    .await
    .unwrap();
}

#[tokio::test]
pub async fn test_metrics_server_listens() {
    let server = TestServer::run(true).await.unwrap();
    tokio::task::spawn_blocking(move || {
        let server_url = &server.server_url;
        let _response = ureq::get(&format!("{server_url}{GetItemsPath}"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        let metrics_server_url = server.metrics_server_url.clone().unwrap();
        let response = ureq::get(&format!("{metrics_server_url}/metrics"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        let line = response
            .lines()
            .find(|line| line.starts_with("http_requests_total"))
            .unwrap();
        let (_, num) = line.split_once(' ').unwrap();
        let num = num.parse::<u8>().unwrap();
        assert!(1 <= num, "{line}");
        assert!(num <= 2, "{line}");
    })
    .await
    .unwrap();
}
