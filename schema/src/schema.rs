// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        lightning_address -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}
