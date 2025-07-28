use crate::models::message;
use crate::models::user::User;
use r2d2::Pool;
use r2d2_sqlite::rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use r2d2_sqlite::rusqlite::params;
use r2d2_sqlite::{SqliteConnectionManager, rusqlite};
use std::error::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct Sqlite {
    pool: Pool<SqliteConnectionManager>,
}

impl Sqlite {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut data_local = dirs::data_local_dir().expect("Could not find local data directory");
        data_local.push("icedm");
        std::fs::create_dir_all(&data_local)?;

        data_local.push("icedm");
        data_local.set_extension("db");

        let manager = SqliteConnectionManager::file(data_local);
        let pool = Pool::new(manager)?;
        let conn = pool.get()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS display_pictures (\
                id INTEGER PRIMARY KEY,\
                picture BLOB NOT NULL,\
                hash TEXT UNIQUE NOT NULL\
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (\
                id INTEGER PRIMARY KEY,\
                email TEXT UNIQUE NOT NULL,\
                personal_message TEXT,\
                display_picture_id INTEGER,\
                FOREIGN KEY (display_picture_id) REFERENCES display_pictures (id)\
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (\
                id INTEGER PRIMARY KEY,\
                sender TEXT NOT NULL,\
                receiver TEXT,\
                is_nudge BOOL NOT NULL,\
                text TEXT NOT NULL,\
                bold BOOL NOT NULL,\
                italic BOOL NOT NULL,\
                underline BOOL NOT NULL,\
                strikethrough BOOL NOT NULL,\
                session_id TEXT\
            )",
            (),
        )?;

        Ok(Self { pool })
    }

    pub fn select_user_emails(&self) -> rusqlite::Result<Vec<String>> {
        if let Ok(conn) = self.pool.get() {
            let mut stmt = conn.prepare("SELECT email FROM users")?;
            let emails = stmt.query_map([], |row| row.get(0));
            return emails?.collect();
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn select_user(&self, email: &str) -> rusqlite::Result<User> {
        if let Ok(conn) = self.pool.get() {
            let mut stmt = conn.prepare(
                "SELECT personal_message, picture, hash FROM users \
                INNER JOIN display_pictures ON users.display_picture_id = display_pictures.id \
                WHERE email = ?1",
            )?;

            let users = stmt.query_map([email], |row| {
                Ok(User {
                    personal_message: row.get(0).ok(),
                    display_picture: row.get(1).ok(),
                })
            });

            return users?.last().ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn select_display_picture(&self, hash: &str) -> rusqlite::Result<Vec<u8>> {
        if let Ok(conn) = self.pool.get() {
            let mut stmt = conn.prepare("SELECT picture FROM display_pictures WHERE hash = ?1")?;
            let picture = stmt.query_map([hash], |row| row.get::<usize, Vec<u8>>(0))?;
            return picture.last().ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn select_messages(
        &self,
        sender1: &str,
        sender2: &str,
    ) -> rusqlite::Result<Vec<message::Message>> {
        if let Ok(conn) = self.pool.get() {
            let mut stmt = conn.prepare(
                "SELECT sender, receiver, is_nudge, text, bold, italic, underline, strikethrough, session_id FROM messages \
                WHERE (sender = ?1 OR receiver = ?1) AND (receiver = ?2 OR sender = ?2)",
            )?;

            let messages = stmt.query_map([sender1, sender2], |row| {
                Ok(message::Message {
                    sender: Arc::new(row.get(0)?),
                    receiver: row.get(1).ok().map(Arc::new),
                    is_nudge: row.get(2)?,
                    text: row.get(3)?,
                    bold: row.get(4)?,
                    italic: row.get(5)?,
                    underline: row.get(6)?,
                    strikethrough: row.get(7)?,
                    session_id: row.get(8).ok().map(Arc::new),
                    color: "0".to_string(),
                    is_history: true,
                })
            });

            return messages?.collect();
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn select_messages_by_session_id(
        &self,
        session_id: &str,
    ) -> rusqlite::Result<Vec<message::Message>> {
        if let Ok(conn) = self.pool.get() {
            let mut stmt = conn.prepare(
                "SELECT sender, receiver, is_nudge, text, bold, italic, underline, strikethrough, session_id FROM messages \
                WHERE session_id = ?1",
            )?;

            let messages = stmt.query_map([session_id], |row| {
                Ok(message::Message {
                    sender: Arc::new(row.get(0)?),
                    receiver: row.get(1).ok().map(Arc::new),
                    is_nudge: row.get(2)?,
                    text: row.get(3)?,
                    bold: row.get(4)?,
                    italic: row.get(5)?,
                    underline: row.get(6)?,
                    strikethrough: row.get(7)?,
                    session_id: row.get(8).ok().map(Arc::new),
                    color: "0".to_string(),
                    is_history: true,
                })
            });

            return messages?.collect();
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn insert_user_if_not_in_db(&self, email: &str) -> rusqlite::Result<()> {
        if let Ok(conn) = self.pool.get() {
            let mut stmt = conn.prepare("SELECT email FROM users WHERE email = ?1")?;
            if stmt.query([email])?.count()? == 0 {
                conn.execute("INSERT INTO users (email) VALUES (?1)", [email])?;
            }
        }

        Ok(())
    }

    pub fn insert_display_picture(
        &self,
        display_picture: &[u8],
        display_picture_hash: &str,
    ) -> rusqlite::Result<()> {
        if let Ok(conn) = self.pool.get() {
            conn.execute(
                "INSERT INTO display_pictures (picture, hash) VALUES (?1, ?2)",
                params![display_picture, display_picture_hash],
            )?;
        }

        Ok(())
    }

    pub fn insert_message(&self, message: &message::Message) -> rusqlite::Result<()> {
        if let Ok(conn) = self.pool.get() {
            conn.execute(
                "INSERT INTO messages (\
                sender,\
                receiver,\
                is_nudge,\
                text,\
                bold,\
                italic,\
                underline,\
                strikethrough,\
                session_id\
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    message.sender,
                    message.receiver,
                    message.is_nudge,
                    message.text,
                    message.bold,
                    message.italic,
                    message.underline,
                    message.strikethrough,
                    message.session_id
                ],
            )?;
        }

        Ok(())
    }

    pub fn update_personal_message(
        &self,
        email: &str,
        personal_message: &str,
    ) -> rusqlite::Result<()> {
        if let Ok(conn) = self.pool.get() {
            conn.execute(
                "UPDATE users SET personal_message = ?1 WHERE email = ?2",
                [personal_message, email],
            )?;
        }

        Ok(())
    }

    pub fn update_user_display_picture(
        &self,
        email: &str,
        display_picture_hash: &str,
    ) -> rusqlite::Result<usize> {
        if let Ok(conn) = self.pool.get() {
            let mut stmt = conn.prepare("SELECT id FROM display_pictures WHERE hash = ?1")?;
            let picture_id = stmt
                .query_map([display_picture_hash], |row| row.get::<usize, usize>(0))?
                .last()
                .ok_or(rusqlite::Error::QueryReturnedNoRows)?;

            return conn.execute(
                "UPDATE users SET display_picture_id = ?1 WHERE email = ?2",
                params![picture_id?, email],
            );
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn delete_user(&self, email: &str) -> rusqlite::Result<()> {
        if let Ok(conn) = self.pool.get() {
            conn.execute("DELETE FROM users WHERE email = ?1", [email])?;
        }

        Ok(())
    }
}
