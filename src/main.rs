//! A proxy server for the Curseforge API.
//! 
//! Curseforge has decided to restrict their API with authentification keys, which is bad news for developers
//! that do not have a single centralized point of API access, but instead ship applications using the CF api
//! to users.
//! 
//! This implements a proxy server that does not use authentification itself - Every request made to this server
//! is passed through mostly unchanged to the CF api, except for a few things:
//! - The `HOST` header is set to `api.curseforge.com`, otherwise CF will not accept requests
//! - An api key is added.
//! 
//! In order to prevent abuse of the api key which is used in every request, this proxy server rate limits per IP.

use std::env;
use std::convert::Infallible;
use std::net::{SocketAddr, IpAddr};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use dotenv::dotenv;
use governor::{RateLimiter, Quota, Jitter};
use hyper::header::{HeaderValue, HeaderName};
use hyper::http::uri::{Authority, Scheme};
use hyper::server::conn::AddrStream;
use hyper::{Body, Client, Request, Response, Server, Uri};
use hyper::service::{make_service_fn, service_fn};
use lazy_static::lazy_static;
use tokio;

lazy_static! {
    /// The CF api key used to authenticate requests. Read from the `CF_API_KEY` env variable.
    static ref CF_API_KEY: String = env::var("CF_API_KEY").expect("Expected CF_API_KEY to contain a cf api key");

    /// The port this proxy is running at. Read from the `PORT` env variable.
    static ref PORT: u16 = env::var("PORT").unwrap_or(String::from("3000"))
        .parse::<u16>().expect("Expected PORT environment variable to contain a number");

    /// How many requests per secs are allowed per ip. Read from the `REQ_LIMIT_PER_SEC` env variable.
    static ref REQ_LIMIT_PER_SEC: u32 = env::var("REQ_LIMIT_PER_SEC").unwrap_or(String::from("6"))
        .parse::<u32>().expect("Expected REQ_LIMIT_PER_SEC env var to contain a number");
}

/// Converts a request to this server into a request that can be made against the Curseforge API.
fn get_proxy_request(mut req: Request<Body>) -> Request<Body> {

    // Set authority part of URL to the Curseforge API & scheme to HTTPS
    let mut uri_parts = req.uri_mut().clone().into_parts();
    uri_parts.authority = Some(Authority::from_static("api.curseforge.com"));
    uri_parts.scheme = Some(Scheme::HTTPS);
    *req.uri_mut() = Uri::from_parts(uri_parts).unwrap();

    // Set HOST header, otherwise CF will reject requests
    req.headers_mut().insert(HeaderName::from_static("host"), HeaderValue::from_static("api.curseforge.com"));

    // Set authentification header
    req.headers_mut().insert("x-api-key", HeaderValue::from_str(&CF_API_KEY[..]).unwrap());

    req
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = SocketAddr::from(([127, 0, 0, 1], *PORT));

    // Init the rate limiter in an ARC so it can be shared across requests
    let rate_limit_quota = Quota::per_second(NonZeroU32::new(*REQ_LIMIT_PER_SEC).expect("Expected req limit to not be null"));
    let limiter = RateLimiter::<IpAddr, _, _>::keyed(rate_limit_quota);
    let bucket = Arc::new(limiter);

    let service = make_service_fn(|socket: &AddrStream| {

        let remote_addr = socket.remote_addr().ip();
        let bucket = Arc::clone(&bucket);

        async move {

            // Wait until the rate limiter allows this request
            bucket.until_key_ready_with_jitter(&remote_addr, Jitter::up_to(Duration::from_secs(1))).await;
            if let Err(_) = bucket.check_key(&remote_addr) {
                println!("[{:>15}] <!> Rate limit was hit", remote_addr.to_string());
            }

            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| async move {

                // Get new CF api request from current request
                let proxy_req = get_proxy_request(req);

                // Init HTTPS client
                let https = hyper_tls::HttpsConnector::new();
                let client = Client::builder().build::<_, Body>(https);
                let uri = proxy_req.uri().clone();

                // Do request & send back response
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

    println!("<-> Server starting at port {}", *PORT);

    // Run until end of time
    if let Err(e) = server.await {
        eprintln!("<!> Server error: {}", e);
    }
}
