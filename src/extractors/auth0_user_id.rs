use actix_web::{dev, error, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};

use jwt::dangerous_insecure_decode;

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0UserId {
  pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0UserClaims {
  pub sub: String,
}

impl FromRequest for Auth0UserId {
  type Error = actix_web::Error;
  // type Error = error::InternalError<String>;
  type Future = Ready<Result<Self, Self::Error>>;
  type Config = ();

  fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
    // let access_token_header_val = req.headers().get("authorization").unwrap(); // TODO: Get rid of this unwrap!
    let access_token_header_val = req
      .headers()
      .get("authorization")
      .ok_or_else(|| "Missing authorization header");

    match access_token_header_val {
      Ok(athv) => {
        let access_token_str = athv
          .to_str()
          .unwrap();
        let access_token: Vec<&str> = access_token_str
          .split_whitespace()
          .collect();

        // Okay to do dangerous_unsafe_decode here because the user has already verified in middleware.
        let decoded_token = dangerous_insecure_decode::<Auth0UserClaims>(
          access_token
            .get(1)
            .unwrap(),
        )
        .ok()
        .unwrap();

        ok(Auth0UserId {
          id: decoded_token
            .claims
            .sub
            .clone(),
        })
      },
      Err(e) => err(error::ErrorUnauthorized(e)),
    }
  }
}
