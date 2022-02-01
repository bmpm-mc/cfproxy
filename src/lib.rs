use std::convert::Infallible;
use std::env;
use std::net::IpAddr;
use hyper::header::{HeaderValue, HeaderName};
use hyper::http::uri::{Authority, Scheme};
use hyper::{Body, Client, Request, Response, Uri};
use lazy_static::lazy_static;

lazy_static! {
    /// The CF api key used to authenticate requests. Read from the `CF_API_KEY` env variable.
    static ref CF_API_KEY: String = env::var("CF_API_KEY").expect("Expected CF_API_KEY to contain a cf api key");
}

/// Converts a request to this server into a request that can be made against the Curseforge API.
/// 
/// Modifies the request by
/// - replacing the base url with https://api.curseforge.com
/// - setting the host to api.curseforge.com
/// - adding the API key read from the env variable
fn get_proxy_req(mut req: Request<Body>) -> Request<Body> {

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

/// Forwards the request to the CF API and returns the API's response.
/// 
/// Request gets mutated with [`get_proxy_request`], Response gets returned directly.
/// `remote_addr` is only used for logging.
pub async fn proxy_request_to_cf(req: Request<Body>, remote_addr: &IpAddr) -> Result<Response<Body>, Infallible> {
    // Get new CF api request from current request
    let proxy_req = get_proxy_req(req);

    // Init HTTPS client
    let https = hyper_tls::HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);
    let uri = proxy_req.uri().clone();

    // Do request & send back response
    match client.request(proxy_req).await {
        Ok(resp) => {
            println!("[{}] <-> {} => {}", remote_addr.to_string(), uri.path(), resp.status().as_str());
            Ok::<_, Infallible>(resp)
        }
        Err(err) => {
            eprintln!("[{}] <!> {} failed: {:#?}", remote_addr.to_string(), uri.path(), err);
            Ok::<_, Infallible>(Response::builder()
                .status(500)
                .body(Body::from("Proxy Server Error while reading request"))
                .unwrap()
            )
        }
    }
}