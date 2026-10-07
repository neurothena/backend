use diesel::{Selectable, deserialize::Queryable, prelude::Insertable};

#[derive(Selectable, Queryable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}