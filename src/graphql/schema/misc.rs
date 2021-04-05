// use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};
// use super::query::Query;

// pub trait Misc {
//   fn api_version() -> String;
// }

// #[graphql_object]
// impl Query {
//     fn api_version() -> String {
//         "1.0".to_string()
//     }
// }
// pub trait Misc {
//   fn api_version() -> String;
// }

// #[graphql_object]
// impl Misc for Query {
//     fn api_version() -> String {
//         "1.0".to_string()
//     }
// }