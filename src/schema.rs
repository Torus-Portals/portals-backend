table! {
    blocks (id) {
        id -> Uuid,
        block_type -> Text,
        portal_id -> Uuid,
        portal_view_id -> Uuid,
        egress -> Text,
        bbox -> Array<Int4>,
        data -> Jsonb,
        created_at -> Timestamptz,
        created_by -> Uuid,
        updated_at -> Timestamptz,
        updated_by -> Uuid,
    }
}

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
        name -> Text,
        org -> Uuid,
        owners -> Array<Uuid>,
        vendors -> Array<Uuid>,
        created_at -> Timestamptz,
        created_by -> Uuid,
        updated_at -> Timestamptz,
        updated_by -> Uuid,
    }
}

table! {
    portalviews (id) {
        id -> Uuid,
        portal_id -> Uuid,
        name -> Text,
        egress -> Text,
        access -> Text,
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
    blocks,
    orgs,
    portals,
    portalviews,
    users,
);
