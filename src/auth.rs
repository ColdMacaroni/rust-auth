use axum::async_trait;
use axum_login::UserId;
use axum_login::{AuthUser, AuthnBackend};
use sqlx::{Executor, Row, SqlitePool};
use std::fmt::Debug;

pub const BCRYPT_COST: u32 = 12;

// Could have more fields, and be able to be constructed From an sqlx row.
// Actually is it ok to clone if it has that many fields? Might want to keep a smaller substruct
// for this if that's a concern.
#[derive(Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub pw_hash: Vec<u8>,
}

impl AuthUser for User {
    type Id = u64;

    fn id(&self) -> Self::Id {
        self.id
    }

    // Returns something to verify the session is valid.
    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash
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

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub pw_hash: String,
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

#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = User;

    #[doc = " Credential type used for authentication."]
    type Credentials = Credentials;

    #[doc = " An error which can occur during authentication and authorization."]
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        _creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // sqlx::query("SELECT * FROM user WHERE username = ? AND password_hash = ?")
        // .execute(&self.pool).await?;

        let row = self
            .pool
            .fetch_one("SELECT * FROM user WHERE username = ? AND password_hash = ?")
            .await?;

        println!("[authenticate] Received row: {:#?}", row.columns());

        Ok(None)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        println!("[get_user] Received id: {:?}", user_id);

        Ok(None)
    }
}
