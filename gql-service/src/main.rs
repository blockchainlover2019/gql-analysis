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
use nonzero::nonzero;
use std::sync::Arc;
use std::env;

mod schema;
mod bookstore;

async fn graphql_handler(
    schema: Extension<Arc<AppSchema>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
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