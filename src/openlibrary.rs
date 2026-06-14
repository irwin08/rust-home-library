use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug,Deserialize)]
pub struct BookInfo {
    pub title: Option<String>,
    pub authors: Option<Vec<Author>>,
    pub publish_date: Option<String>,
    pub publishers: Option<Vec<Publisher>>
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Publisher {
    pub name: String,
}

pub async fn lookup_isbn(isbn: &str) -> Option<BookInfo> {
    let url = format!(
	"https://openlibrary.org/api/books?bibkeys=ISBN:{}&format=json&jscmd=data",
	isbn
    );

    let res = reqwest::get(&url).await.ok()?;
    
    let mut map : HashMap<String, BookInfo> = res.json().await.ok()?;
    let key = format!("ISBN:{}", isbn);
    map.remove(&key)
}
