use juniper::{FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::services::db::connection_content_service::get_connection_content;

use super::{
  blocks::{empty_block::EmptyBlock, table_block::TableBlock, xy_chart_block::XYChartBlock},
  Query,
};

use crate::graphql::context::GQLContext;

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, EnumString, Display)]
pub enum ContentTypes {
  #[strum(serialize = "TableBlock")]
  #[graphql(name = "TableBlock")]
  TableBlock,

  #[strum(serialize = "TextBlock")]
  #[graphql(name = "TextBlock")]
  TextBlock,

  #[strum(serialize = "CellsBlock")]
  #[graphql(name = "CellsBlock")]
  CellsBlock,

  #[strum(serialize = "XYChartBlock")]
  #[graphql(name = "XYChartBlock")]
  XYChartBlock,
}

#[derive(Debug, GraphQLUnion, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
pub enum ConnectionContentData {
  Empty(EmptyBlock),
  TableBlock(TableBlock),
  XYChartBlock(XYChartBlock),
}

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
#[graphql(Context = GQLContext)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionContent {
  pub source_id: Uuid,

  pub connection_id: Uuid,

  pub content_type: ContentTypes,

  pub content_data: ConnectionContentData,
}

impl Query {
  pub async fn connection_content_impl(
    ctx: &GQLContext,
    block_id: Uuid,
  ) -> FieldResult<Vec<ConnectionContent>> {
    let local_pool = ctx.pool.clone();
    let conn_content = get_connection_content(local_pool, &ctx.auth0_user_id, block_id).await?;

    Ok(conn_content)
  }
}
