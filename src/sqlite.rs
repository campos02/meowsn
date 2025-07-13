use crate::models::user::User;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2_sqlite::rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
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
            "CREATE TABLE IF NOT EXISTS users (\
                id INTEGER PRIMARY KEY,\
                email TEXT UNIQUE NOT NULL,\
                personal_message TEXT,\
                display_picture BLOB\
            )",
            (),
        )?;

        Ok(Self { pool })
    }

    pub fn select_user_emails(&self) -> Vec<String> {
        let mut emails = Vec::new();
        if let Ok(conn) = self.pool.get() {
            if let Ok(mut stmt) = conn.prepare("SELECT email FROM users") {
                let users = stmt.query_map([], |row| {
                    Ok(User {
                        email: row.get(0)?,
                        personal_message: None,
                    })
                });

                if let Ok(users) = users {
                    for user in users {
                        if let Ok(user) = user {
                            emails.push(user.email);
                        }
                    }
                }
            }
        }

        emails
    }

    pub fn select_personal_message(&self, email: &str) -> String {
        let mut psm = String::new();
        if let Ok(conn) = self.pool.get() {
            if let Ok(mut stmt) =
                conn.prepare("SELECT personal_message FROM users WHERE email = ?1")
            {
                let users = stmt.query_map([email], |row| {
                    Ok(User {
                        email: String::new(),
                        personal_message: row.get(0)?,
                    })
                });

                if let Ok(users) = users {
                    if let Some(Ok(user)) = users.last() {
                        if let Some(personal_message) = user.personal_message {
                            psm = personal_message;
                        }
                    }
                }
            }
        }

        psm
    }

    pub fn insert_user_if_not_in_db(&self, email: &str) {
        if let Ok(conn) = self.pool.get() {
            if let Ok(mut stmt) = conn.prepare("SELECT email FROM users WHERE email = ?1") {
                if let Ok(rows) = stmt.query([email]) {
                    if let Ok(count) = rows.count() {
                        if count == 0 {
                            let _ = conn.execute("INSERT INTO users (email) VALUES (?1)", [email]);
                        }
                    }
                }
            }
        }
    }

    pub fn update_personal_message(&self, email: &str, personal_message: &str) {
        if let Ok(conn) = self.pool.get() {
            let _ = conn.execute(
                "UPDATE users SET personal_message = ?1 WHERE email = ?2",
                [personal_message, email],
            );
        }
    }

    pub fn delete_user(&self, email: &str) {
        if let Ok(conn) = self.pool.get() {
            let _ = conn.execute("DELETE FROM users WHERE email = ?1", [email]);
        }
    }
}
