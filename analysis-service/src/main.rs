use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum_wasi::{GraphQLRequest, GraphQLResponse};
use crate::schema::{AppSchema, Mutation, Query};
use axum::{
    extract::{Extension, RequestParts, TypedHeader},
    headers::{ authorization::Bearer, Authorization, CONTENT_TYPE },
    http::{Request, StatusCode, Method},
    middleware::{self, Next},
    response::{self, IntoResponse, Response},
    routing::get,
    Router,
    handler::Handler,
    body::Bytes,
};
use governor::{clock::FakeRelativeClock, Quota, RateLimiter};
use nonzero::nonzero;
use std::sync::Arc;
use std::env;
use http_body::combinators::UnsyncBoxBody;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use hyper::*;

mod schema;

async fn graphql_handler(
    schema: Extension<Arc<AppSchema>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    println!("here");
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}


async fn auth_middleware<B>(
  request: Request<B>,
  next: Next<B>,
) -> Result<Response, StatusCode>
where
  B: Send,
{
  // running extractors requires a `RequestParts`
  let mut request_parts = RequestParts::new(request);

  // `TypedHeader<Authorization<Bearer>>` extracts the auth token but
  // `RequestParts::extract` works with anything that implements `FromRequest`
  let auth = request_parts.extract::<TypedHeader<Authorization<Bearer>>>()
      .await
      .map_err(|_| StatusCode::UNAUTHORIZED)?;

  if !token_is_valid(auth.token()) {
      return Err(StatusCode::UNAUTHORIZED);
  }

  // get the request back so we can run `next`
  //
  // `try_into_request` will fail if you have extracted the request body. We
  // know that `TypedHeader` never does that.
  //
  // see the `consume-body-in-extractor-or-middleware` example if you need to
  // extract the body
  let request = request_parts.try_into_request().expect("body extracted");

  Ok(next.run(request).await)
}


fn token_is_valid(_token: &str) -> bool {
    println!("token {}", _token);
    true
}

async fn rate_limiter_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: Send,
{
    let clock = FakeRelativeClock::default();
    let rate_limiter = RateLimiter::direct_with_clock(Quota::per_second(nonzero!(5u32)), &clock);

    // Check if the request is within the rate limit
    match rate_limiter
        .check()
        .map_err(|outcome| outcome.quota().burst_size().get())
    {
        Ok(_) => Ok(next.run(request).await),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

fn get_router() -> Router {
    let schema = Arc::new(
        Schema::build(Query, Mutation, EmptySubscription)
            .limit_depth(5)
            .limit_complexity(50)
            .finish(),
    );
    Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(rate_limiter_middleware))
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let app = get_router();
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = axum::Server::bind(&addr).serve(app.into_make_service());
    println!("Server listening on {}", addr);
    server.await.unwrap();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> anyhow::Result<()> {
  use std::os::wasi::io::FromRawFd;
  
  tracing_subscriber::fmt::init();

  let std_listener = unsafe { std::net::TcpListener::from_raw_fd(3) };
  std_listener.set_nonblocking(true).expect("Cannot set non-blocking");

  axum::Server::from_tcp(std_listener)
      .unwrap()
      .serve(get_router().into_make_service()).await
      .unwrap();

  println!("server is running");
  Ok(())
}

fn platform() -> String {
  let mut name = env::consts::ARCH.to_string();
  if env::consts::OS.len() > 0 {
      name = format!("{}-{}", name, env::consts::OS);
  }
  name
}

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

pub async fn get_request(
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

pub async fn post_request<T: Serialize>(
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

#[cfg(test)]
pub mod tests {
    use super::*;


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