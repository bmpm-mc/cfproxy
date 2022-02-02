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
use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Server};
use hyper::service::{make_service_fn, service_fn};
use lazy_static::lazy_static;
use tokio;

lazy_static! {
    /// The port this proxy is running at. Read from the `PORT` env variable.
    static ref PORT: u16 = env::var("PORT").unwrap_or(String::from("3000"))
        .parse::<u16>().expect("Expected PORT environment variable to contain a number");

    /// How many requests per secs are allowed per ip. Read from the `REQ_LIMIT_PER_SEC` env variable.
    static ref REQ_LIMIT_PER_SEC: u32 = env::var("REQ_LIMIT_PER_SEC").unwrap_or(String::from("6"))
        .parse::<u32>().expect("Expected REQ_LIMIT_PER_SEC env var to contain a number");
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], *PORT));

    // Init the rate limiter in an ARC so it can be shared across requests
    let rate_limit_quota = Quota::per_second(NonZeroU32::new(*REQ_LIMIT_PER_SEC).expect("Expected req limit to not be null"));
    let limiter = RateLimiter::<IpAddr, _, _>::keyed(rate_limit_quota);
    let bucket = Arc::new(limiter);

    let service = make_service_fn(move |socket: &AddrStream| {

        let remote_addr = socket.remote_addr().ip();
        let bucket = Arc::clone(&bucket);

        async move {

            let service = service_fn(move |req: Request<Body>| {

                let bucket = Arc::clone(&bucket);

                async move {
                    // Wait until the rate limiter allows this request
                    let remote_addr = cfproxy::get_real_ip_addr(&req, &remote_addr);
                    bucket.until_key_ready_with_jitter(&remote_addr, Jitter::up_to(Duration::from_secs(1))).await;
                    if let Err(_) = bucket.check_key(&remote_addr) {
                        println!("[{}] <!> Rate limit was hit", remote_addr.to_string());
                    }
                    cfproxy::proxy_request_to_cf(req, &remote_addr).await
                }
            });

            // Pass the request to the service handler
            Ok::<_, Infallible>(service)
        }
    });

    let server = Server::bind(&addr).serve(service);

    println!("<-> Server starting at port {}", *PORT);

    // Run until end of time
    if let Err(e) = server.await {
        eprintln!("<!> Server error: {}", e);
    }
}
