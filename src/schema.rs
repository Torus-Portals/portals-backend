table! {
    orgs (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        auth0id -> Varchar,
        name -> Varchar,
        nickname -> Varchar,
        email -> Varchar,
        orgs -> Array<Int4>,
    }
}

allow_tables_to_appear_in_same_query!(
    orgs,
    users,
);
