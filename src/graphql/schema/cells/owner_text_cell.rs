use juniper::GraphQLObject;

#[derive(GraphQLObject, Debug, Serialize, Deserialize)]
pub struct OwnerTextCell {
  text: String,
}
