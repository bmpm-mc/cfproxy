use std::env;
use std::convert::Infallible;
use std::net::{SocketAddr, IpAddr};
use std::sync::Arc;
use std::time::Duration;
use dotenv::dotenv;
use governor::{RateLimiter, Quota, Jitter};
use hyper::server::conn::AddrStream;
use hyper::{Body, Client, Request, Response, Server, Uri};
use hyper::service::{make_service_fn, service_fn};
use lazy_static::lazy_static;
use tokio;
use nonzero_ext::nonzero;

lazy_static! {
    static ref CF_API_KEY: String = env::var("CF_API_KEY").expect("Expected CF_API_KEY to contain a cf api key");
    static ref PORT_ENV: String = env::var("PORT").unwrap_or(String::from("3000"));
    static ref PORT: u16 = PORT_ENV.parse::<u16>().expect("Expected PORT environment variable to contain a number");
}

fn get_proxy_request(req: Request<Body>) -> Request<Body> {
    let uri = Uri::builder()
        .scheme("https")
        .authority("api.curseforge.com")
        .path_and_query(req.uri().path_and_query().unwrap().clone())
        .build()
        .unwrap();

    Request::builder()
        .method(req.method())
        .uri(uri)
        .header("x-api-key", &CF_API_KEY[..])
        .header("Accept", "application/json")
        .body(req.into_body())
        .unwrap()
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = SocketAddr::from(([127, 0, 0, 1], *PORT));

    let rate_limit_quota = Quota::per_second(nonzero!(6u32));
    let limiter = RateLimiter::<IpAddr, _, _>::keyed(rate_limit_quota);
    let bucket = Arc::new(limiter);

    let service = make_service_fn(|socket: &AddrStream| {
        let remote_addr = socket.remote_addr().ip();
        let bucket = Arc::clone(&bucket);
        async move {
            if let Err(_) = bucket.check_key(&remote_addr) {
                println!("[{:>15}] <!> Rate limit was hit", remote_addr.to_string());
            }
            bucket.until_key_ready_with_jitter(&remote_addr, Jitter::up_to(Duration::from_secs(1))).await;
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| async move {
                let https = hyper_tls::HttpsConnector::new();
                let client = Client::builder().build::<_, Body>(https);
                let proxy_req = get_proxy_request(req);
                let uri = proxy_req.uri().clone();
                if let Ok(resp) = client.request(proxy_req).await {
                    println!("[{:>15}] <-> {} => {}", remote_addr.to_string(), uri.path(), resp.status().as_str());
                    Ok::<_, Infallible>(resp)
                } else {
                    eprintln!("[{:>15}] <!> {} failed", remote_addr.to_string(), uri.path());
                    Ok::<_, Infallible>(Response::builder().status(500).body(Body::from("Proxy Server Error while reading request")).unwrap())
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    println!("Server starting at port {}", *PORT);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
