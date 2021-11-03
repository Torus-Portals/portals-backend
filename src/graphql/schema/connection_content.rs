use juniper::{FieldResult, GraphQLEnum, GraphQLObject};
use strum_macros::{Display, EnumString};
use std::str::FromStr;

use crate::services::db::connection_content_service::get_connection_content;

use super::{Query};

use crate::graphql::context::GQLContext;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum ContentTypes {
  #[strum(serialize = "TableBlock")]
  #[graphql(name = "TableBlock")]
  TableBlock
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionContent {
 pub source_id: Uuid,

 pub content_type: ContentTypes,

 pub content_data: String,
}

impl Query {
  pub async fn connection_content_impl(ctx: &GQLContext, block_id: Uuid) -> FieldResult<Vec<ConnectionContent>> {
    let local_pool = ctx.pool.clone();
    let conn_content = get_connection_content(local_pool, &ctx.auth0_user_id, block_id).await?;

    Ok(conn_content)
  }
}