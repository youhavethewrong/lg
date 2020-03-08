#![feature(async_closure)]
use futures::stream::{self, StreamExt};
use hyper::{Client, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use tokio::fs;

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
    println!("Main screen turn on.");

    let config_path = "config.toml";
    let mut config_file = File::open(config_path)?;
    let mut config_buffer = String::new();
    config_file.read_to_string(&mut config_buffer)?;
    let decoded: Targets = toml::from_str(&config_buffer)?;
    let targets = decoded.targets;
    println!("Fetching {} targets.", targets.len());
    let target_stream = stream::iter(targets);

    let fut = target_stream.for_each_concurrent(4, |target| async move {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let parsed_url = target.url.parse::<Uri>().unwrap();
        let resp = client.get(parsed_url).await.unwrap();
        match resp.status() {
            StatusCode::OK => {
                let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
                fs::write(&target.filename, body).await.unwrap();
                println!("Wrote {}.", target.filename);
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
                println!("Wrote {}.", target.filename);
            }
            other => println!(
                "Unable to retrieve '{}' because status code was '{:?}'.",
                target.url, other
            ),
        }
    });

    fut.await;

    println!("Done!");
    Ok(())
}
