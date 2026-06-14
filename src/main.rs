mod db;
mod openlibrary;

use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

type SharedDb = Arc<Mutex<Connection>>;

#[derive(Deserialize)]
struct IsbnRequest {
    isbn: String,
}

#[derive(Serialize)]
struct LookupResponse {
    title: Option<String>,
    author: Option<String>,
    year: Option<String>,
    publisher: Option<String>,
}

#[derive(Deserialize)]
struct SaveRequest {
    isbn: String,
    title: String,
    author: String,
    year: String,
    publisher: String,
    status: String,
    ownership: String,
    tags: String,
    shelf: String,
    notes: String,
}

#[tokio::main]
async fn main() {
    let conn = Connection::open("books.db").expect("Failed to open database");
    db::init_db(&conn).expect("Failed to initialize database");
    let shared_db: SharedDb = Arc::new(Mutex::new(conn));

    let app = Router::new()
        .route("/lookup", post(lookup_handler))
        .route("/save", post(save_handler))
        .fallback_service(ServeDir::new("static"))
        .with_state(shared_db);

    let tls_config = RustlsConfig::from_pem_file("cert.pem", "key.pem")
    .await
    .expect("Failed to load TLS cert");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on https://0.0.0.0:3000");
    axum_server::bind_rustls(addr, tls_config)
    .serve(app.into_make_service())
    .await
    .unwrap();

    if let Some(book) = openlibrary::lookup_isbn("9780521596794").await {
	println!("Title: {:?}", book.title);
        println!("Authors: {:?}", book.authors);
        println!("Year: {:?}", book.publish_date);
        println!("Publishers: {:?}", book.publishers);
    } else {
	println!("Book not found.");
    }
}

async fn lookup_handler(Json(req): Json<IsbnRequest>) -> Json<LookupResponse> {
    let book = openlibrary::lookup_isbn(&req.isbn).await;

    match book {
	Some(b) => Json(LookupResponse {
	    title: b.title,
	    author: b.authors.map(|a| {
		a.into_iter()
		    .map(|x| x.name)
		    .collect::<Vec<_>>()
		    .join(", ")
	    }),
	    year: b.publish_date,
	    publisher: b.publishers.map(|p| {
		p.into_iter()
		    .map(|x| x.name)
		    .collect::<Vec<_>>()
		    .join(", ")
	    }),
	}),
	None => Json(LookupResponse {
	    title: None,
	    author: None,
	    year: None,
	    publisher: None,
	}),
    }
}

async fn save_handler(
    State(db): State<SharedDb>,
    Json(req): Json<SaveRequest>,
) -> &'static str {
    let conn = db.lock().unwrap();
    conn.execute(
	"INSERT INTO books (isbn, title, author, year, publisher, status, ownership, tags, shelf, notes)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
	(
	    &req.isbn, &req.title, &req.author, &req.year, &req.publisher, &req.status, &req.ownership, &req.tags, &req.shelf, &req.notes,
	)
    ).expect("Failed to insert book");

    "OK"
}
