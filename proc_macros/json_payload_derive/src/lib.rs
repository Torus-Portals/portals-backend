extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

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
      type Error = error::JsonPayloadError;
      type Future = Box<dyn Future<Item = Self, Error = error::JsonPayloadError>>;
      type Config = ();

      fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        Box::new(
          dev::JsonBody::<Self>::new(req, payload, None)
        )
      }
    }
  };

  gen.into()
}