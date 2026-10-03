// @generated automatically by Diesel CLI.

diesel::table! {
    user_jti (jti) {
        jti -> Uuid,
        user_email -> Text,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        email -> Text,
        username -> Text,
        password_hash -> Text,
        id -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    user_jti,users,);
