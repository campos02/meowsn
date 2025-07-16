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

    pub fn select_user_emails(&self) -> Vec<String> {
        let mut emails = Vec::new();
        if let Ok(conn) = self.pool.get() {
            if let Ok(mut stmt) = conn.prepare("SELECT email FROM users") {
                let users = stmt.query_map([], |row| {
                    Ok(User {
                        email: row.get(0)?,
                        personal_message: None,
                        display_picture: None,
                    })
                });

                if let Ok(users) = users {
                    for user in users.flatten() {
                        emails.push(user.email);
                    }
                }
            }
        }

        emails
    }

    pub fn select_user(&self, email: &str) -> Option<User> {
        if let Ok(conn) = self.pool.get() {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT personal_message, picture, hash FROM users \
                INNER JOIN display_pictures ON users.display_picture_id = display_pictures.id \
                WHERE email = ?1",
            ) {
                let users = stmt.query_map([email], |row| {
                    Ok(User {
                        email: String::new(),
                        personal_message: row.get(0)?,
                        display_picture: row.get(1)?,
                    })
                });

                if let Ok(users) = users {
                    if let Some(Ok(user)) = users.last() {
                        return Some(user);
                    }
                }
            }
        }

        None
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

    pub fn update_user_display_picture(
        &self,
        email: &str,
        display_picture_hash: &str,
    ) -> rusqlite::Result<usize> {
        if let Ok(conn) = self.pool.get() {
            if let Ok(mut stmt) = conn.prepare("SELECT id FROM display_pictures WHERE hash = ?1") {
                let picture_id =
                    stmt.query_map([display_picture_hash], |row| row.get::<usize, usize>(0));

                if let Ok(ids) = picture_id {
                    if let Some(Ok(id)) = ids.last() {
                        return conn.execute(
                            "UPDATE users SET display_picture_id = ?1 WHERE email = ?2",
                            params![id, email],
                        );
                    }
                }
            }
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn update_user_with_new_display_picture(
        &self,
        email: &str,
        display_picture: &[u8],
        display_picture_hash: &str,
    ) {
        if let Ok(conn) = self.pool.get() {
            let _ = conn.execute(
                "INSERT INTO display_pictures (picture, hash) VALUES (?1, ?2)",
                params![display_picture, display_picture_hash],
            );

            if let Ok(mut stmt) = conn.prepare("SELECT id FROM display_pictures WHERE hash = ?1") {
                let picture_id =
                    stmt.query_map([display_picture_hash], |row| row.get::<usize, usize>(0));

                if let Ok(ids) = picture_id {
                    if let Some(Ok(id)) = ids.last() {
                        let _ = conn.execute(
                            "UPDATE users SET display_picture_id = ?1 WHERE email = ?2",
                            params![id, email],
                        );
                    }
                }
            }
        }
    }

    pub fn delete_user(&self, email: &str) {
        if let Ok(conn) = self.pool.get() {
            let _ = conn.execute("DELETE FROM users WHERE email = ?1", [email]);
        }
    }
}
