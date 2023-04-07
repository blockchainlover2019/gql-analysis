#![allow(unused_imports, unused_variables, dead_code)]

use async_graphql::{EmptySubscription};

use std::sync::Arc;
use std::env;
use std::net::SocketAddr;

use hyper::Client;
use hyper::server::conn::Http;
use hyper::service::{ service_fn };
use tokio::net::TcpListener;
use tower::limit::{RateLimitLayer};

use graphql_depth_limit::QueryDepthAnalyzer;
use graphql_parser::parse_query;

mod config;
use config::Config;

fn rate_limiter_check() -> bool {
  true
}

fn token_is_valid(_token: &str) -> bool {
  _token == "Bearer test_token"
}
/*
pub async fn post_to_gql_service(
  body: hyper::Body,
) -> hyper::Response<hyper::Body> {
  use hyper::{Body, Method, Request, Response};
  let client = Client::new();
  println!("post_to_gql_service: ");
  let req = hyper::Request::builder()
      .method(Method::POST)
      .uri("http://0.0.0.0:27017")
      .header(hyper::header::CONTENT_TYPE, "application/json")
      .header(hyper::header::AUTHORIZATION, "Bearer test_token")
      .body(body)
      .expect("request builder");

  let res = client.request(req).await.unwrap();
  println!("res: {:?}", res);
  //Response::builder().body("some string".into()).unwrap()
  res
}*/

#[derive(Debug, serde::Deserialize)]
struct GQuery { query: String }

#[cfg(feature = "standalone")]
pub fn get_query_from_bytes(bytes: &[u8]) -> String {
  let query_value = serde_json::from_slice::<GQuery>(&bytes).unwrap();
  query_value.query.to_string()
}

#[cfg(not(feature = "standalone"))]
pub fn get_query_from_bytes(bytes: &[u8]) -> String {
  let query = std::str::from_utf8(&bytes).unwrap();
  query.to_string()
}

async fn check_depth_limit(
  bytes: &[u8],
  max_depth: u32
) -> bool {
  let query = get_query_from_bytes(bytes);
  println!("query: {:?}", query);
  let depth = QueryDepthAnalyzer::new(&query, vec![], |_a, _b| true).unwrap();
  match depth.verify(max_depth as usize) {
    Ok(depth) => { println!("depth: {:?}", depth);  true},
    Err(_) => false
  }
}

async fn request_handler(
  req: hyper::Request<hyper::Body>,
  config: Config
) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
  use hyper::{Body, Method, Request, Response};

    match (req.method(), req.uri().path()) {
      (&Method::POST, "/") => {
        if rate_limiter_check() {
          let token = req.headers()
            .get(hyper::header::AUTHORIZATION)
            .unwrap()
            .to_str()
            .unwrap();
          println!("token {:?}", token);
          match token_is_valid(token) {
            true => {
              let body = req.into_body();
              let bytes = hyper::body::to_bytes(body).await.unwrap();
              println!("bytes {:?}", bytes);
              if !check_depth_limit(&bytes, config.max_depth).await {
                println!("depth limit");
                Ok(Response::builder().status(400).body("Depth Limit Error".into()).unwrap())
              } else {
                Ok(Response::builder().status(200).body("Good".into()).unwrap())
                // Ok(post_to_gql_service(bytes.into()).await)
              }
            }
            _ => Ok(Response::builder().status(400).body("Authorization Error".into()).unwrap())
          }
          
        } else {
          Ok(Response::builder().status(400).body("Rate limit Error".into()).unwrap())
        }
      },    
      _ => {
        Ok(Response::new(Body::from("😡 try again")))
      } 
  }
}



#[tokio::main(flavor = "current_thread")]
pub async fn main() -> anyhow::Result<()> {
  use std::convert::Infallible;
  
  let sec_config = Config::read_config();

  let addr = SocketAddr::from(([127, 0, 0, 1], 3005));

  let listener = TcpListener::bind(addr).await?;
  println!("Listening on http://{}", addr);

  loop {
    let (stream, _) = listener.accept().await?;
    let service_handler = service_fn(move |req| async move {
      request_handler(req, sec_config).await
    });
    
    tokio::task::spawn(async move {
        if let Err(err) = Http::new().serve_connection(stream, service_handler).await {
          println!("Error serving connection: {:?}", err);
        } 
    });
  }
}

fn platform() -> String {
  let mut name = env::consts::ARCH.to_string();
  if env::consts::OS.len() > 0 {
      name = format!("{}-{}", name, env::consts::OS);
  }
  name
}