use hyper::{Client, Uri};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Main screen turn on.");

    let client = Client::new();

    let ip_fut = async {
        let resp = client
            .get(Uri::from_static("http://httpbin.org/ip"))
            .await
            .unwrap();
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        fs::write("ip.txt", body).await
    };
    let headers_fut = async {
        let resp = client
            .get(Uri::from_static("http://httpbin.org/headers"))
            .await
            .unwrap();
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        fs::write("headers.txt", body).await
    };

    let (ip, headers) = futures::try_join!(ip_fut, headers_fut)?;
    println!("{:?} and {:?}", ip, headers);
    Ok(())
}
