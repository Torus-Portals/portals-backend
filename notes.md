pub fn create_user(
  new_user_json: web::Json<NewUser>, 
  pool: web::Data<Pool>
) -> impl Future<Item = HttpResponse, Error = Error> {
  // println!("Hello!! {:#?}", &new_user);
  // let new_user = NewUser::from_json()
  // fut_ok(HttpResponse::Created().json(new_user_json.0))

  ...
}

https://cetra3.github.io/blog/face-detection-with-actix-web/