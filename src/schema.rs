// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        email -> Text,
        username -> Text,
        password_hash -> Text,
        id -> Int4,
    }
}
