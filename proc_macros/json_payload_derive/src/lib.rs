extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

extern crate futures;

use proc_macro::TokenStream;

#[proc_macro_derive(JSONPayload)]
pub fn json_payload_derive(input: TokenStream) -> TokenStream {
  let ast = syn::parse(input)
    .expect("Unable to parse json_payload_derive input");

  json_payload_macro(&ast)
}

fn json_payload_macro(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;
  let gen = quote! {
    impl FromRequest for #name {
      type Error = error::Error;
      type Future = futures::future::LocalBoxFuture<'static, Result<Self, Self::Error>>;
      type Config = ();

      fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        use futures::future::{ FutureExt, LocalBoxFuture };

        let req2 = req.clone();

        dev::JsonBody::<Self>::new(req, payload, None)
          .map(move |res| match res {
            Err(e) => {
              Err(e.into())
            }
            Ok(data) => Ok(data),
            // Ok(data) => Ok(actix_web::web::Json(data)),
          })
        .boxed_local()
      }
    }
  };

  gen.into()
}