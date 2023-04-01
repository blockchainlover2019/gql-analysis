#![allow(unused_imports, unused_variables, dead_code)]

use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use crate::schema::{AppSchema, Mutation, Query};
use axum::{
    extract::{Extension, TypedHeader},
    headers::authorization::Bearer,
    headers::Authorization,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{self, IntoResponse, Response},
    routing::get,
    Router,
    handler::Handler,
};
use governor::{clock::FakeRelativeClock, Quota, RateLimiter};
use nonzero::nonzero;
use std::sync::Arc;
use std::env;

mod schema;
mod bookstore;

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

fn get_router() -> Router {
    let schema = Arc::new(
        Schema::build(Query, Mutation, EmptySubscription).finish(),
    );
    Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let app = get_router();
    let addr = "0.0.0.0:27017".parse().unwrap();
    let server = axum::Server::bind(&addr).serve(app.into_make_service());
    println!("Server listening on {}", addr);
    server.await.unwrap();
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use axum::{
        body::Bytes,
        http::{
            header::{AUTHORIZATION, CONTENT_TYPE},
            Method, Request,
        },
    };
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
