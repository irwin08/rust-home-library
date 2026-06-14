use rusqlite::{Connection, Result};

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
	"CREATE TABLE IF NOT EXISTS books (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            isbn        TEXT NOT NULL,
            title       TEXT,
            author      TEXT,
            year        TEXT,
            publisher   TEXT,
            status      TEXT DEFAULT 'unread',
            ownership   TEXT DEFAULT 'owned',
            tags        TEXT,
            shelf       TEXT,
            notes  
	)"
    )?;
    Ok(())
}
