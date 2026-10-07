use chrono::{DateTime, NaiveDateTime};
use diesel::{Selectable, deserialize::Queryable, prelude::Insertable};
use uuid::Uuid;

#[derive(Selectable, Queryable)]
#[diesel(table_name = crate::schema::user_jti)]
pub struct UserJti {
    pub jti: Uuid,
    pub user_email: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_jti)]
pub struct NewUserJti {
    pub jti: Uuid,
    pub user_email: String,
    pub expires_at: NaiveDateTime,
}

impl NewUserJti {
    pub fn from_timestamp(jti: Uuid, user_email: String, exp_timestamp: i64) -> Self {
        let expires_at_naive: NaiveDateTime = DateTime::from_timestamp(exp_timestamp, 0)
            .map(|dt| dt.naive_utc())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        Self {
            jti,
            user_email,
            expires_at: expires_at_naive
        }
    }
}