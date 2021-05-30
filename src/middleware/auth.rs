use actix_web::dev::ServiceRequest;
use actix_web::{Error};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::{AuthenticationError};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
  exp: usize,
  aud: Vec<String>,
}

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
  let key = req.app_data::<DecodingKey>().unwrap();

  let mut validation = Validation {
    algorithms: vec![Algorithm::HS256],
    leeway: 120,
    ..Validation::default()
  };

  // TODO: Remove these hardcoded audiences. Pull from env?
  validation.set_audience(&[
    // "http://localhost:8088",
    // "https://torus-rocks.auth0.com/userinfo",
    "a7781863-554e-4849-9be4-020aa067a2cc"
  ]);

  match decode::<Claims>(credentials.token(), &key, &validation) {
    Ok(_) => Ok(req),
    Err(_err) => {
      // TODO: figure out how to better handle this error;
      let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);
      Err(AuthenticationError::from(config).into())
    }
  }
}

// use std::pin::Pin;
// use std::task::{Context, Poll};
// use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey, errors::ErrorKind };

// use actix_service::{ Service, Transform };
// use futures::future::{ok, Ready };
// use futures::Future;
// use actix_web::{
//   dev,
//   Error,
//   HttpResponse
// };

// use dev:: {ServiceRequest, ServiceResponse };

// // use jwt::{ Validation, decode, Algorithm, errors::ErrorKind };

// #[derive(Debug, Serialize, Deserialize)]
// struct Claims {
//   sub: String,
//   exp: usize,
//   aud: Vec<String>
// }

// pub struct Auth;

// pub struct AuthDer (pub Vec<u8>);

// impl<S, Req> Transform<S, Req> for Auth
// where
//   S: Service<Req>,
//   // S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//   // S::Future: 'static,
//   // B: 'static,
// {
//   type Response = S::Response;
//   type Error = S::Error;
//   type InitError = S::Error;
//   type Transform = AuthMiddleware<S>;
//   type Future = Ready<Result<Self::Transform, Self::InitError>>;

//   fn new_transform(&self, service: S) -> Self::Future {
//     ok(AuthMiddleware { service })
//   }
// }

// pub struct AuthMiddleware<S> {
//   service: S,
// }

// impl<S, Req> Service<Req> for AuthMiddleware<S>
// where
//   S: Service<Req>,
//   Req: ServiceRequest,
//   // S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//   // S::Future: 'static,
//   // B: 'static,
// {
//   // type Request = Req;
//   type Response = S::Response;
//   type Error = S::Error;
//   type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//   fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//     self.service.poll_ready(cx)
//   }

//   fn call(&self, req: Req) -> Self::Future {
//     match req.headers().get("authorization") {
//       Some(access_token_header_val) => {
//         let key = req.app_data::<DecodingKey>().unwrap();

//         let access_token_str = access_token_header_val.to_str().unwrap();
//         let access_token: Vec<&str> = access_token_str.split_whitespace().collect();

//         let mut validation = Validation {
//           algorithms: vec![Algorithm::RS256],
//           leeway: 120,
//           ..Validation::default()
//         };

//         // TODO: Remove these hardcoded audiences. Pull from env?
//         validation.set_audience(&[
//             "http://localhost:8088",
//             "https://torus-rocks.auth0.com/userinfo",
//         ]);
//         // if cfg!(feature = "local_dev") {
//         //   validation.set_audience(&[
//         //       "http://localhost:8088",
//         //       "https://torus-rocks.auth0.com/userinfo",
//         //       "https://portals-backend-1.caprover.portals-dev.rocks/",
//         //   ]);
//         // } else {
//         //   validation.set_audience(&[
//         //       "https://portals-backend-1.caprover.portals-dev.rocks/",
//         //   ]);
//         // }

//         match decode::<Claims>(access_token.get(1).unwrap(), &key, &validation) {
//           Ok(_) => {
//             let fut = self.service.call(req);

//             Box::pin(async move {
//               let res = fut.await?;

//               Ok(res)
//             })
//           },
//           Err(err) => {
//             match err.kind() {
//               // TODO: Needs better error handling, but this works for now.
//               ErrorKind::InvalidToken => {
//                 Box::pin(
//                   ok(
//                     req.into_response(
//                       HttpResponse::Unauthorized()
//                       .body(String::from(err.to_string()))
//                       // .finish()
//                       .into_body()
//                     )
//                   )
//                 )
//               },
//               // ErrorKind::InvalidToken => Box::pin(ok(req.into_response(HttpResponse::Unauthorized().finish().into_body()))),
//               // ErrorKind::InvalidToken => Either::B(ServiceResponse::new(req, HttpResponse::Unauthorized().finish().into_body())),
//               // ErrorKind::InvalidToken => panic!("Token is invalid"),
//               // ErrorKind::InvalidIssuer => panic!("Issuer is invalid"),
//               // ErrorKind::InvalidRsaKey => panic!("InvalidRsaKey"),
//               // ErrorKind::InvalidSignature => panic!("Invalid Signature"),
//               // ErrorKind::InvalidAudience => panic!("InvalidAudience"),
//               // ErrorKind::InvalidAlgorithm => panic!("InvalidAlgorithm"),
//               // ErrorKind::ImmatureSignature => panic!("ImmatureSignature"),
//               // ErrorKind::ExpiredSignature => panic!("ExpiredSignature"),
//               // ErrorKind::InvalidSubject => panic!("InvalidSubject"),
//               // ErrorKind::InvalidEcdsaKey => panic!("InvalidEcdsaKey"),
//               // ErrorKind::InvalidAlgorithmName => panic!("InvalidAlgorithmName"),
//               // ErrorKind::Json(_) => panic!("Json"),
//               // ErrorKind::Base64(a) => {
//               //   println!("a: {:#?}", a.clone());
//               //   panic!("Base64")
//               // },
//               // ErrorKind::Crypto(_) => panic!("Crypto"),
//               // ErrorKind::Utf8(_) => panic!("Utf8"),
//               // ErrorKind::__Nonexhaustive => panic!("dunno..."),
//               // _ => Either::B(fut_ok(req.error_response(err.into()))),
//               _ => {
//                 Box::pin(
//                   ok(
//                     req.into_response(
//                       HttpResponse::Unauthorized()
//                       .body(String::from(err.to_string()))
//                       // .finish()
//                       .into_body()
//                     )
//                   )
//                 )
//               },
//             }
//           }
//         }
//       },
//       None => Box::pin(ok(req.into_response(
//         HttpResponse::Found()
//         .finish()
//         .into_body())))
//     }
//   }
// }
