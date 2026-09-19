// @generated automatically by Diesel CLI.

diesel::table! {
    users (email) {
        email -> Text,
        username -> Text,
        password_hash -> Text,
    }
}
