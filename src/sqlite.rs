use crate::models::user::User;
use r2d2::Pool;
use r2d2_sqlite::rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use r2d2_sqlite::rusqlite::params;
use r2d2_sqlite::{SqliteConnectionManager, rusqlite};
use std::error::Error;

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
                picture BLOB,\
                hash TEXT\
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

        Ok(Self { pool })
    }

    pub fn select_user_emails(&self) -> rusqlite::Result<Vec<String>> {
        let mut emails = Vec::new();
        if let Ok(conn) = self.pool.get() {
            let mut stmt = conn.prepare("SELECT email FROM users")?;
            let users = stmt.query_map([], |row| {
                Ok(User {
                    email: row.get(0)?,
                    personal_message: None,
                    display_picture: None,
                })
            });

            for user in users?.flatten() {
                emails.push(user.email);
            }
        }

        Ok(emails)
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
                    email: String::new(),
                    personal_message: row.get(0)?,
                    display_picture: row.get(1)?,
                })
            });

            return users?.last().ok_or(rusqlite::Error::QueryReturnedNoRows)?;
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
