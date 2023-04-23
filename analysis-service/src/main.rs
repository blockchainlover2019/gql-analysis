#![allow(unused_imports, unused_variables, dead_code)]

use async_graphql::{EmptySubscription};

use std::env;
use std::net::SocketAddr;
use std::{collections::HashMap, num::NonZeroU32, sync::Arc};

use hyper::Client;
use hyper::server::conn::Http;
use hyper::service::{ service_fn };
use hyper::StatusCode;
use hyper::{Body, Method, Request, Response};

use tokio::net::TcpListener;
use tower::limit::{RateLimitLayer};

use graphql_depth_limit::QueryDepthAnalyzer;
use graphql_parser::parse_query;
use governor::{Quota, RateLimiter};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use serde::{Deserialize, Serialize};

mod config;
use config::Config;

fn rate_limiter_check() -> bool {
  true
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,
    sub: String,
    exp: usize,
}

fn token_is_valid(_token: &str) -> bool {
  true
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

#[no_mangle]
async fn get_query_depth(
  bytes: &[u8],
  max_depth: u32
) -> usize {
  let query = get_query_from_bytes(bytes);
  println!("query: {:?}", query);
  let depth = QueryDepthAnalyzer::new(&query, vec![], |_a, _b| true).unwrap();
  depth.verify(max_depth as usize).unwrap()
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
    match (req.method(), req.uri().path()) {
      (&Method::POST, "/") => {
        let body = req.into_body();
        let bytes = hyper::body::to_bytes(body).await.unwrap();
        if !check_depth_limit(&bytes, config.max_depth).await {
          Ok(Response::builder().status(StatusCode::BAD_REQUEST).body("Depth Limit Error".into()).unwrap())
        } else {
          Ok(Response::builder().status(StatusCode::OK).body("Good".into()).unwrap())
          // Ok(post_to_gql_service(bytes.into()).await)
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
  
  let q = Quota::per_minute(NonZeroU32::new(sec_config.rate_limit_per_minute).unwrap());
  let gov: Arc<RateLimiter<String, _, _>> = Arc::new(governor::RateLimiter::dashmap(q));
  
  loop {    
    let gov_loop = Arc::clone(&gov);
    let key: Arc<Hmac<Sha256>> = Arc::new(Hmac::new_from_slice(b"big secret").unwrap());
    println!("JWT example: {}", make_jwt(key.as_ref()));

    // POST / takes form with a token and performs rate limiting and openid verification on it.  
    let (stream, _) = listener.accept().await?;
    let service_handler = service_fn(move |req| {
      let gov = Arc::clone(&gov_loop);
      let key = Arc::clone(&key);
      async move {
        let token = match req.headers().get(hyper::header::AUTHORIZATION) {
          Some(_token) => Some(_token.to_str().unwrap_or("Bearer default_token")),
          None => None
        };
        if token.is_none() {
          return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body("Auth token is expected".into()).unwrap());
        }
        let jwt_token = token.unwrap().strip_prefix("Bearer ").unwrap();
        let claims: Result<Claims, _> = jwt_token.verify_with_key(key.as_ref());
        match claims {
            Ok(claims) => match gov.clone().check_key(&claims.sub) {
                Ok(_) => request_handler(req, sec_config).await,
                Err(_) => Ok(Response::builder().status(StatusCode::TOO_MANY_REQUESTS).body("Rate Limit Error".into()).unwrap())
            },
            Err(e) => {
                println!("bad token {:#?}", e);
                Ok(Response::builder().status(StatusCode::FORBIDDEN).body("Authorization Error".into()).unwrap())
            }
        }
      }
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

fn make_jwt(key: &Hmac<Sha256>) -> String {
  use jwt::SignWithKey;
  let c = Claims {
    aud: "".to_string(),
    sub: "me".to_string(),
    exp: 0,
  };
  c.sign_with_key(key).unwrap()
}