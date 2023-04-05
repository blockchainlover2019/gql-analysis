use async_graphql::{ID, Context, EmptySubscription, Object, Result, Schema};
use async_graphql::SimpleObject;

use crate::bookstore::BookStore;
pub struct Query;

#[derive(Clone)]
pub struct Book {
    pub id: ID,
    pub name: String,
    pub author: String,
}

#[derive(Default)]
struct PubInfo;

#[derive(Default)]
struct Publisher;

#[derive(Default)]
struct Location;

#[derive(Default)]
struct Address;


#[Object]
impl Book {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn author(&self) -> &str {
        &self.author
    }

    async fn desc(&self) -> PubInfo {
        PubInfo::default()
    }
}

#[Object]
impl PubInfo {
    async fn publisher(&self) -> Publisher {
        Publisher::default()
    }
}

#[Object]
impl Publisher {
    async fn location(&self) -> Location {
        Location::default()
    }
}

#[Object]
impl Location {
    async fn flat_address(&self) -> &str {
        "hello, depth is 5 here"
    }
    async fn address(&self) -> Address {
        Address::default()
    }
}

#[Object]
impl Address {
    async fn hello(&self) -> &str {
        "hello"
    }
}

#[Object]
impl Query {
    async fn hello(&self) -> &str {
        "Hello, world!"
    }

    #[graphql(entity)]
    async fn find_book_by_id(&self, id: ID) -> Book {
      BookStore::get_book_by_id(id.as_str()).unwrap_or(Book {
        id: "default".into(),
        name: "name".to_string(),
        author: "author".to_string()
      })
    }
    async fn books(&self, ctx: &Context<'_>) -> Vec<Book> {
        BookStore::get_books()
    }

    #[graphql(complexity = 5)]
    async fn value(&self) -> i32 {
        5
        // todo!()
    }

    #[graphql(complexity = "count * child_complexity")]
    async fn values(&self, count: usize) -> i32 {
        20
        // todo!()
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
