table! {
    orgs (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamptz,
        created_by -> Uuid,
        updated_at -> Timestamptz,
        updated_by -> Uuid,
    }
}

table! {
    portals (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        created_by -> Uuid,
        updated_at -> Timestamptz,
        updated_by -> Uuid,
    }
}

table! {
    users (id) {
        id -> Uuid,
        auth0id -> Text,
        name -> Text,
        nickname -> Text,
        email -> Text,
        orgs -> Array<Uuid>,
        created_at -> Timestamptz,
        created_by -> Uuid,
        updated_at -> Timestamptz,
        updated_by -> Uuid,
    }
}

allow_tables_to_appear_in_same_query!(
    orgs,
    portals,
    users,
);
