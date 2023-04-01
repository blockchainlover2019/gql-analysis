use crate::schema::Book;

pub struct BookStore;

impl BookStore {
    pub fn get_book_by_id(id: &str) -> Option<Book> {
        let books: Vec<Book> = Self::get_books();
        books.iter().find(|b| b.id == id).cloned()
    }
    pub fn get_books() -> Vec<Book> {
        vec![
            Book {
                id: "12430-fga-3234".into(),
                name: "The Editor".to_string(),
                author: "Steven Rowley".to_string(),
            },
            Book {
                id: "959334-gfgj-3234".into(),
                name: "Harry Potter".to_string(),
                author: "J. K. Rowling".to_string(),
            },
            Book {
                id: "234232342".into(),
                name: "About Rust".to_string(),
                author: "Rustacean".to_string(),
            },
            Book {
                id: "asdfasfasdf".into(),
                name: "About Graphql".to_string(),
                author: "graphql".to_string(),
            },
            Book {
                id: "3243fadfasdfasd".into(),
                name: "About Apollo Router".to_string(),
                author: "apollo".to_string(),
            },
        ]
    }
}
