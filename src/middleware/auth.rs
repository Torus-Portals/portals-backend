use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{ Service, Transform };
use futures::future::{ok, Ready };
use futures::Future;
use actix_web::{
  dev,
  Error,
  HttpResponse
};

use dev:: {ServiceRequest, ServiceResponse };

use jwt::{ Validation, decode, Algorithm, errors::ErrorKind };

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
  exp: usize,
  aud: Vec<String>
}

pub struct Auth;

pub struct AuthDer (pub Vec<u8>);

impl<S, B> Transform<S> for Auth
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = AuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(AuthMiddleware { service })
  }
}

pub struct AuthMiddleware<S> {
  service: S,
}

impl<S, B> Service for AuthMiddleware<S>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&mut self, req: ServiceRequest) -> Self::Future {
    match req.headers().get("authorization") {
      Some(access_token_header_val) => {
        let key = req.app_data::<AuthDer>().unwrap();

        let access_token_str = access_token_header_val.to_str().unwrap();
        let acces_token: Vec<&str> = access_token_str.split_whitespace().collect();

        let mut validation = Validation { 
          algorithms: vec![Algorithm::RS256],
          leeway: 120,
          ..Validation::default()
        };

        // TODO: Remove these hardcoded audiences. Pull from env?
        if cfg!(feature = "local_dev") {
          validation.set_audience(&[
              "http://localhost:8088",
              "https://torus-rocks.auth0.com/userinfo",
          ]);
        } else {
          validation.set_audience(&[
              "https://portals-backend-1.caprover.portals-dev.rocks/",
          ]);
        }

        match decode::<Claims>(acces_token.get(1).unwrap(), &key.0, &validation) {
          Ok(_) => {
            let fut = self.service.call(req);

            Box::pin(async move {
              let res = fut.await?;

              Ok(res)
            })
          },
          Err(err) => match err.kind() {
            // TODO: Needs better error handling, but this works for now.
            ErrorKind::InvalidToken => Box::pin(ok(req.into_response(HttpResponse::Unauthorized().finish().into_body()))),
            // ErrorKind::InvalidToken => Either::B(ServiceResponse::new(req, HttpResponse::Unauthorized().finish().into_body())),
            // ErrorKind::InvalidToken => panic!("Token is invalid"),
            // ErrorKind::InvalidIssuer => panic!("Issuer is invalid"),
            // ErrorKind::InvalidRsaKey => panic!("InvalidRsaKey"),
            // ErrorKind::InvalidSignature => panic!("Invalid Signature"),
            // ErrorKind::InvalidAudience => panic!("InvalidAudience"),
            // ErrorKind::InvalidAlgorithm => panic!("InvalidAlgorithm"),
            // ErrorKind::ImmatureSignature => panic!("ImmatureSignature"),
            // ErrorKind::ExpiredSignature => panic!("ExpiredSignature"),
            // ErrorKind::InvalidSubject => panic!("InvalidSubject"),
            // ErrorKind::InvalidEcdsaKey => panic!("InvalidEcdsaKey"),
            // ErrorKind::InvalidAlgorithmName => panic!("InvalidAlgorithmName"),
            // ErrorKind::Json(_) => panic!("Json"),
            // ErrorKind::Base64(a) => {
            //   println!("a: {:#?}", a.clone());
            //   panic!("Base64")
            // },
            // ErrorKind::Crypto(_) => panic!("Crypto"),
            // ErrorKind::Utf8(_) => panic!("Utf8"),
            // ErrorKind::__Nonexhaustive => panic!("dunno..."),
            // _ => Either::B(fut_ok(req.error_response(err.into()))),
            _ => Box::pin(ok(req.into_response(HttpResponse::Unauthorized().finish().into_body()))),
          }
        }
      },
      None => Box::pin(ok(req.into_response(
        HttpResponse::Found()
        .finish()
        .into_body())))
    }
  }
}