use juniper::{graphql_object, GraphQLInputObject, GraphQLObject};

use crate::graphql::context::GQLContext;
use crate::services::integration_service::{SheetsCells, SheetsObject};

use super::Query;

// A Google spreadsheet represented with dimensions (row == major dimension)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoogleRowSheet {
  pub row_dimensions: Vec<String>,
  pub col_dimensions: Vec<String>,
  pub data: Vec<Vec<String>>,
}

#[derive(GraphQLInputObject)]
pub struct IntegrationData {
  pub row_dimension: String,
  pub col_dimension: String,
}

impl From<SheetsObject> for GoogleRowSheet {
  fn from(obj: SheetsObject) -> Self {
    let values = &obj.value_ranges[0].values;
    let col_dimensions = values[0].clone();
    let row_dimensions: Vec<String> = values.iter().skip(1).map(|row| row[0].clone()).collect();

    GoogleRowSheet {
      row_dimensions,
      col_dimensions,
      // TODO: very inefficient copy; redundant once not handling entire spreadsheet anymore?
      data: values.to_vec(),
    }
  }
}

// TODO: implement juniper Scalar for u64?
// Currently working with `users` as row dimensions
#[graphql_object(context = GQLContext)]
impl GoogleRowSheet {
  // Returns all column values, including `users`
  pub async fn col_dimensions(&self) -> &Vec<String> {
    &self.col_dimensions
  }

  // Currently returns all row values (`users` names)
  pub async fn row_dimensions(&self) -> &Vec<String> {
    &self.row_dimensions
  }

  // TODO: retain col and row dimension as metadata?
  pub async fn cell_by_dimensions(&self, data: IntegrationData) -> &String {
    let row_dim = &data.row_dimension;
    let col_dim = &data.col_dimension;

    let row_idx = self
      .row_dimensions
      .iter()
      .position(|s| s == row_dim)
      .expect(&format!("Unable to find row dimension: {}", row_dim));

    let col_idx = self
      .col_dimensions
      .iter()
      .position(|s| s == col_dim)
      .expect(&format!("Unable to find column dimension: {}", col_dim));

    &self.data[row_idx + 1][col_idx]
  }
}

impl Query {
  pub async fn sheet_impl() -> GoogleRowSheet {
    let client = reqwest::Client::new();
    let resp = client
      .get("http://localhost:8088/auth")
      .send()
      .await
      .unwrap();
    let sheets_obj: SheetsObject = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
    let google_sheet: GoogleRowSheet = sheets_obj.into();

    google_sheet
  }
}
