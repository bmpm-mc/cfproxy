#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use cfproxy::proxy_request_to_cf;
    use hyper::{Request, Body, StatusCode};
    use tokio;
    use dotenv::dotenv;

    #[tokio::test]
    async fn it_works() {
        dotenv().ok();
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let req: Request<Body> = Request::builder()
            .method("GET")
            .uri("http://localhost:3000")
            .body(Body::default())
            .unwrap();
        let result = proxy_request_to_cf(req, &ip).await;
        let resp = result.expect("Expected an result");
        let (parts, body) = resp.into_parts();
        let body = hyper::body::to_bytes(body).await.expect("Expected a body");
        let body = String::from_utf8(body.to_vec()).expect("Expected a string body");
        assert_eq!(parts.status, StatusCode::OK);
        assert!(body.starts_with("CurseForge Core"));
    }
}