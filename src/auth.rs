use axum::async_trait;
use axum_login::UserId;
use axum_login::{AuthUser, AuthnBackend};
use sqlx::prelude::FromRow;
use sqlx::{Row, SqlitePool};
use std::fmt::Debug;
use thiserror::Error;

pub const BCRYPT_COST: u32 = 12;

// Could have more fields, and be able to be constructed From an sqlx row.
// Actually is it ok to clone if it has that many fields? Might want to keep a smaller substruct
// for this if that's a concern.
#[derive(Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,

    #[sqlx(rename = "password_hash")]
    pub pw_hash: String,
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    // Returns something to verify the session is valid.
    fn session_auth_hash(&self) -> &[u8] {
        self.pw_hash.as_bytes()
    }
}

// Needed for the AuthUser trait
impl Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Let's not leak the hash :')
        f.debug_struct("User")
            .field("id", &self.id)
            .field("name", &self.username)
            .field("pw_hash", &"Wouldn't you like to know")
            .finish()
    }
}

// TODO: Any way to do it by reference?
#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

// Needed for the AuthUser trait
impl Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Let's not leak the hash :')
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("pw_hash", &"Wouldn't you like to know")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct AuthBackend {
    pub pool: SqlitePool,
}

/* #[derive(Error, Debug)]
struct AuthError(String);
impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
} */

#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // let Some(user): Option<User> = sqlx::query_as("SELECT * FROM user WHERE username = ?")
        //     .bind(creds.username)
        //     .fetch_optional(&self.pool)
        //     .await?
        // else {
        //     return Ok(None);
        // };
        //
        // // Idfk how to do error types ok
        // if bcrypt::verify(creds.password, &user.pw_hash)
        //     .expect("database's hash should be formatted ok")
        // {
        //     Ok(Some(user))
        // } else {
        //     Ok(None)
        // }


        dbg!(
            sqlx::query_as("SELECT * FROM user WHERE username = ? AND password_hash = ?")
                .bind(creds.username)
                .bind(creds.pw_hash)
                .fetch_optional(&self.pool)
                .await
        )
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        println!("[get_user] Received id: {:?}", user_id);

        sqlx::query_as("SELECT * FROM user WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
    }
}
pub type AuthSession = axum_login::AuthSession<AuthBackend>;
