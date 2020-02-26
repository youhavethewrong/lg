#![feature(async_closure)]
use futures::stream::{self, StreamExt};
use hyper::{Client, Uri};
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Main screen turn on.");

    let config_path = "config.toml";
    let mut config_file = File::open(config_path).unwrap();
    let mut config_buffer = String::new();
    config_file.read_to_string(&mut config_buffer).unwrap();
    let decoded: Targets = toml::from_str(&config_buffer).unwrap();
    let targets = decoded.targets;
    println!("Fetching {} targets.", targets.len());
    let mut target_stream = stream::iter(targets);

    while let Some(target) = target_stream.next().await {
        let client = Client::new();
        let parsed_url = target.url.parse::<Uri>().unwrap();
        let resp = client.get(parsed_url).await.unwrap();
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        fs::write(&target.filename, body).await.unwrap();
        println!("Wrote {}.", target.filename);
    }

    println!("Done!");
    Ok(())
}
