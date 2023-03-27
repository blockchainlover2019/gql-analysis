use async_graphql::{Context, EmptySubscription, Object, Result, Schema};

pub struct Query;

#[Object]
impl Query {
    async fn hello(&self) -> &str {
        "Hello, world!"
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn increment(&self, ctx: &Context<'_>, value: i32) -> Result<i32> {
        let new_value = value + 1;
        Ok(new_value)
    }
}

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;