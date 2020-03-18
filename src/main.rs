#![feature(async_closure)]
use futures::stream::{self, StreamExt};
use hyper::{Client, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use tokio::{fs, sync};

mod display;

#[derive(Clone, Debug)]
pub struct RequestResult {
    start: std::time::Instant,
    end: std::time::Instant,
    status: hyper::StatusCode,
    len_bytes: usize,
}

impl RequestResult {
    pub fn duration(&self) -> std::time::Duration {
        self.end - self.start
    }
}

#[derive(Debug, Deserialize)]
struct Target {
    pub url: String,
    pub filename: String,
}

#[derive(Debug, Deserialize)]
struct Targets {
    pub targets: Vec<Target>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let (tx, rx) = sync::mpsc::channel(100);

    tokio::spawn(
        display::Monitor {
            report_receiver: rx,
            start,
            fps: 30,
        }
        .monitor(),
    );

    let config_path = "config.toml";
    let mut config_file = File::open(config_path)?;
    let mut config_buffer = String::new();
    config_file.read_to_string(&mut config_buffer)?;
    let decoded: Targets = toml::from_str(&config_buffer)?;
    let targets = decoded.targets;
    let transmitters = (0..=targets.len()).map(|_| tx.clone()).collect::<Vec<_>>();
    let combined_stream = targets.iter().zip(transmitters).collect::<Vec<_>>();

    let fut =
        stream::iter(combined_stream).for_each_concurrent(2, |(target, mut txer)| async move {
            let start = std::time::Instant::now();
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, hyper::Body>(https);
            let parsed_url = target.url.parse::<Uri>().unwrap();
            let resp = client.get(parsed_url).await.unwrap();
            let end = std::time::Instant::now();
            let r = RequestResult {
                start: start,
                end: end,
                status: resp.status(),
                len_bytes: 0,
            };

            match resp.status() {
                StatusCode::OK => {
                    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
                    fs::write(&target.filename, body).await.unwrap();
                    txer.send(Ok(r)).await.unwrap()
                }
                StatusCode::FOUND => {
                    let location = &resp.headers()["Location"];
                    let follow_resp = client
                        .get(location.to_str().unwrap().parse().unwrap())
                        .await
                        .unwrap();
                    let body = hyper::body::to_bytes(follow_resp.into_body())
                        .await
                        .unwrap();
                    fs::write(&target.filename, body).await.unwrap();
                    txer.send(Ok(r)).await.unwrap()
                }
                _ => txer.send(Ok(r)).await.unwrap(),
            }
        });

    fut.await;
    Ok(())
}
