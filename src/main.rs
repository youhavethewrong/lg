use hyper::{Client, Uri};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let client = Client::new();

    let ip_fut = async {
        let resp = client
            .get(Uri::from_static("http://httpbin.org/ip"))
            .await?;
        let ok = hyper::body::to_bytes(resp.into_body()).await;
        println!("Finished ip!");
        ok
    };
    let headers_fut = async {
        let resp = client
            .get(Uri::from_static("http://httpbin.org/headers"))
            .await?;
        let ok = hyper::body::to_bytes(resp.into_body()).await;
        println!("Finished headers!");
        ok
    };

    // Wait on both them at the same time:
    let (ip, headers) = futures::try_join!(ip_fut, headers_fut)?;
    println!("{:?} and {:?}", ip, headers);
    Ok(())
}
