mod db;
mod openlibrary;

use axum::{
    extract::{Path, State},
    response::{Json, IntoResponse, Response},
    http::header,
    routing::{get, post, delete},
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
    duplicate: bool,
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

#[derive(Serialize)]
struct BookRecord {
    id: i64,
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
        .route("/export", get(export_handler))
        .route("/books", get(list_books_handler))
        .route("/books/{id}", delete(delete_handler).put(update_handler))
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

async fn lookup_handler(
    State(db): State<SharedDb>,
    Json(req): Json<IsbnRequest>,
) -> Json<LookupResponse> {
    let book = openlibrary::lookup_isbn(&req.isbn).await;

    let duplicate = {
	let conn = db.lock().unwrap();
	let count: i64 = conn.query_row(
	    "SELECT count(*) FROM books WHERE isbn = ?1",
	    [&req.isbn],
	    |row| row.get(0),
	).unwrap_or(0);
	count > 0
    };

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
	    duplicate,
	}),
	None => Json(LookupResponse {
	    title: None,
	    author: None,
	    year: None,
	    publisher: None,
	    duplicate,
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

async fn export_handler(State(db): State<SharedDb>) -> impl IntoResponse {
    let conn = db.lock().unwrap();

    let mut stmt = conn.prepare(
	"SELECT isbn, title, author, year, publisher, status, ownership, tags, shelf, notes FROM books"
    ).expect("Failed to prepare statement");

    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&["isbn", "title", "author", "year", "publisher", "status", "ownership", "tags", "shelf", "notes"])
        .expect("Failed to write header");

    let rows = stmt.query_map([], |row| {
	Ok((
	    row.get::<_, String>(0)?,
	    row.get::<_, String>(1)?,
	    row.get::<_, String>(2)?,
	    row.get::<_, String>(3)?,
	    row.get::<_, String>(4)?,
	    row.get::<_, String>(5)?,
	    row.get::<_, String>(6)?,
	    row.get::<_, String>(7)?,
	    row.get::<_, String>(8)?,
	    row.get::<_, String>(9)?,
	))
    }).expect("Failed to query");

    for row in rows {
	let r = row.expect("Failed to read row");
	wtr.write_record(&[r.0, r.1, r.2, r.3, r.4, r.5, r.6, r.7, r.8, r.9])
	    .expect("Failed to write row");
    }

    let csv_data = wtr.into_inner().expect("Failed to finalize CSV");

    Response::builder()
        .header(header::CONTENT_TYPE, "text/csv")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"books.csv\"")
        .body(axum::body::Body::from(csv_data))
        .unwrap()
}

async fn list_books_handler(State(db): State<SharedDb>) -> Json<Vec<BookRecord>> {
    let conn = db.lock().unwrap();

    let mut stmt = conn.prepare(
	"SELECT id, isbn, title, author, year, publisher, status, ownership, tags, shelf, notes FROM books ORDER BY id DESC"
    ).expect("Failed to prepare statement");

    let books = stmt.query_map([], |row| {
	Ok(BookRecord {
	    id: row.get(0)?,
	    isbn: row.get(1)?,
	    title: row.get(2)?,
	    author: row.get(3)?,
	    year: row.get(4)?,
	    publisher: row.get(5)?,
	    status: row.get(6)?,
	    ownership: row.get(7)?,
	    tags: row.get(8)?,
	    shelf: row.get(9)?,
	    notes: row.get(10)?,
	})
    }).expect("Failed to query")
	.filter_map(|r| r.ok())
	.collect();

    Json(books)
}

async fn delete_handler(
    State(db): State<SharedDb>,
    Path(id): Path<i64>,
) -> &'static str {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM books WHERE id = ?1", [id])
        .expect("Failed to delete book");
    "OK"
}

async fn update_handler(
    State(db): State<SharedDb>,
    Path(id): Path<i64>,
    Json(req): Json<SaveRequest>,
) -> &'static str {
    let conn = db.lock().unwrap();
    conn.execute(
	"UPDATE books SET isbn=?1, title=?2, author=?3, year=?4, publisher=?5,
         status=?6, ownership=?7, tags=?8, shelf=?9, notes=?10 WHERE id=?11",
	(
	    &req.isbn, &req.title, &req.author, &req.year, &req.publisher,
	    &req.status, &req.ownership, &req.tags, &req.shelf, &req.notes,
	    id,
	),
    ).expect("Failed to update book");
    "OK"
}
