#![allow(unused_imports, unused_variables, dead_code)]

use async_graphql::{EmptySubscription, Schema};
use crate::schema::{Mutation, Query};
use std::sync::Arc;
use std::env;
use hyper::Client;
use std::net::SocketAddr;
use hyper::server::conn::Http;
use hyper::service::{ service_fn };
use tokio::net::TcpListener;

use tower::limit::{RateLimitLayer};

mod schema;

fn rate_limiter_check() -> bool {


    // Check if the request is within the rate limit
    /*rate_limiter
        .check()
        .map_err(|outcome| outcome.quota().burst_size().get())
        .is_ok()*/
        true
}

fn token_is_valid(_token: &str) -> bool {
  _token == "Bearer test_token"
}

pub async fn post_to_gql_service(
  body: hyper::Body,
) -> hyper::Response<hyper::Body> {
  use hyper::{Body, Method, Request, Response};
  let client = Client::new();

  let req = hyper::Request::builder()
      .method(Method::POST)
      .uri("http://0.0.0.0:27017")
      .header(hyper::header::CONTENT_TYPE, "application/json")
      .header(hyper::header::AUTHORIZATION, "Bearer test_token")
      .body(body)
      .expect("request builder");
  
  let res = client.request(req).await.unwrap();
  //Response::builder().body("some string".into()).unwrap()
  res
}
 
async fn request_handler(
  req: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
  use hyper::{Body, Method, Request, Response};

  let schema = Arc::new(Schema::build(Query, Mutation, EmptySubscription)
        .limit_depth(5)
        .limit_complexity(50)
        .finish());
  println!("request arrived");
  match (req.method(), req.uri().path()) {
    (&Method::POST, "/") => {
      if rate_limiter_check() {
        /*let token = req.headers()
          .get(hyper::header::AUTHORIZATION)
          .unwrap()
          .to_str()
          .unwrap();
        println!("token {:?}", token);
        if token_is_valid(token) {
          Ok(post_to_gql_service(req.into_body()).await)
        } else {
          Ok(Response::builder().status(400).body("Authorization Error".into()).unwrap())
        }*/
        Ok(post_to_gql_service(req.into_body()).await)
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

  let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

  let listener = TcpListener::bind(addr).await?;
  println!("Listening on http://{}", addr);

  loop {
    let (stream, _) = listener.accept().await?;
    let allowed_rps = 1;
    let rate_limiter = RateLimitLayer::new(allowed_rps, std::time::Duration::from_secs(1));
    /*let service = tower::ServiceBuilder::new()
            .layer(rate_limiter.clone())
            .service_fn(request_handler);*/

    tokio::task::spawn(async move {
        if let Err(err) = Http::new().serve_connection(stream, service_fn(request_handler)).await {
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use http_body::combinators::UnsyncBoxBody;
    use serde::{de::DeserializeOwned, Serialize};
    use serde_json::json;

    pub async fn send_request(
        router: &Router,
        request: Request<hyper::Body>,
    ) -> hyper::Response<UnsyncBoxBody<Bytes, axum::Error>> {
        router
            .clone()
            .oneshot(request)
            .await
            .expect("failed to send oneshot request")
    }

    pub async fn get(
        router: &Router,
        uri: impl AsRef<str>,
    ) -> hyper::Response<UnsyncBoxBody<Bytes, axum::Error>> {
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri.as_ref())
            .body(hyper::Body::empty())
            .expect("failed to build GET request");
        send_request(router, request).await
    }

    pub async fn post<T: Serialize>(
        router: &Router,
        uri: impl AsRef<str>,
        body: &T,
    ) -> hyper::Response<UnsyncBoxBody<Bytes, axum::Error>> {
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri.as_ref())
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, "Bearer test_token")
            .body(
                serde_json::to_vec(body)
                    .expect("failed to serialize POST body")
                    .into(),
            )
            .expect("failed to build POST request");
        send_request(router, request).await
    }

    pub async fn deserialize_response_body<T>(
        response: hyper::Response<UnsyncBoxBody<Bytes, axum::Error>>,
    ) -> T
    where
        T: DeserializeOwned,
    {
        let bytes = hyper::body::to_bytes(response.into_body())
            .await
            .expect("failed to read response body into bytes");
        serde_json::from_slice::<T>(&bytes).expect("failed to deserialize response")
    }

    #[tokio::test]

    async fn try_post() {
        const uri: &'static str = "https://localhost:8000";
        let request_body = /*json!({
          "query": r#"
            {
              user(id: "1") {
                id,
                username
              }
            }
          "#
        });*/
        json! ({
          "query": r#"{
            hello
          }"#
        });
        let response = post(&get_router(), "/", &request_body).await;

        println!("response {:?}", response);
        assert_eq!(response.status(), 200);

        println!(
            "response_body {:?}",
            deserialize_response_body::<serde_json::Value>(response).await
        );
    }
}
