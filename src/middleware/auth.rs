use actix_service::{ Service, Transform };
use futures::future::{ ok as fut_ok, FutureResult, Either };
use futures::{ Future, Poll };
use actix_web::{
  dev,
  Error,
  HttpResponse
};

use dev:: {ServiceRequest, ServiceResponse };

use jwt;
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
  type Future = FutureResult<Self::Transform, Self::InitError>;

  fn new_transform(&self, service: S) -> Self::Future {
    fut_ok(AuthMiddleware { service })
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
  // type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;
  type Future = Either<S::Future, FutureResult<Self::Response, Self::Error>>;

  fn poll_ready(&mut self) -> Poll<(), Self::Error> {
    self.service.poll_ready()
  }

  fn call(&mut self, req: ServiceRequest) -> Self::Future {

    match req.headers().get("authorization") {
      Some(access_token_header_val) => {
        let key = req.app_data::<AuthDer>().unwrap();

        let access_token_str = access_token_header_val.to_str().unwrap();
        let acces_token: Vec<&str> = access_token_str.split_whitespace().collect();

        // let validation = jwt::Validation::new(jwt::Algorithm::RS256);
        let mut validation = jwt::Validation { 
          algorithms: vec![jwt::Algorithm::RS256],
          ..jwt::Validation::default()
        };
        validation.set_audience(&[
            "http://localhost:8088",
            "https://torus-rocks.auth0.com/userinfo",
        ]);

        let token_data = match jwt::decode::<Claims>(acces_token.get(1).unwrap(), &key.0, &validation) {
          Ok(c) => c,
          Err(err) => match err.kind() {
            jwt::errors::ErrorKind::InvalidToken => panic!("Token is invalid"),
            jwt::errors::ErrorKind::InvalidIssuer => panic!("Issuer is invalid"),
            jwt::errors::ErrorKind::InvalidRsaKey => panic!("InvalidRsaKey"),
            jwt::errors::ErrorKind::InvalidSignature => panic!("Invalid Signature"),
            jwt::errors::ErrorKind::InvalidAudience => panic!("InvalidAudience"),
            jwt::errors::ErrorKind::InvalidAlgorithm => panic!("InvalidAlgorithm"),
            jwt::errors::ErrorKind::ImmatureSignature => panic!("ImmatureSignature"),
            jwt::errors::ErrorKind::ExpiredSignature => panic!("ExpiredSignature"),
            jwt::errors::ErrorKind::InvalidSubject => panic!("InvalidSubject"),
            jwt::errors::ErrorKind::InvalidEcdsaKey => panic!("InvalidEcdsaKey"),
            jwt::errors::ErrorKind::InvalidAlgorithmName => panic!("InvalidAlgorithmName"),
            jwt::errors::ErrorKind::Json(_) => panic!("Json"),
            jwt::errors::ErrorKind::Base64(a) => {
              println!("a: {:#?}", a.clone());
              panic!("Base64")
            },
            jwt::errors::ErrorKind::Crypto(_) => panic!("Crypto"),
            jwt::errors::ErrorKind::Utf8(_) => panic!("Utf8"),
            jwt::errors::ErrorKind::__Nonexhaustive => panic!("dunno..."),
            // _ => panic!("some other errors"),
          }
        };

        println!("token data? {:#?}", token_data);

        Either::A(self.service.call(req))
      },
      None => Either::B(fut_ok(req.into_response(
        HttpResponse::Found()
        .finish()
        .into_body())))
    }
  }
}