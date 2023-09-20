// @generated automatically by Diesel CLI.

diesel::table! {
    appjobs (id) {
        id -> Int4,
        service -> Varchar,
        route -> Varchar,
        job_id -> Uuid,
    }
}

diesel::table! {
    escalonjobs (id) {
        id -> Uuid,
        status -> Varchar,
        schedule -> Varchar,
        since -> Nullable<Timestamp>,
        until -> Nullable<Timestamp>,
    }
}

diesel::joinable!(appjobs -> escalonjobs (job_id));

diesel::allow_tables_to_appear_in_same_query!(
    appjobs,
    escalonjobs,
);
