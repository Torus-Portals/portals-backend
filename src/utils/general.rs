pub fn env_var(var_name: &str) -> String {
  let var_str = std::env::var(var_name).expect(&format!("Unable to get {} env var", var_name));

  // Might be good to put this behind a local/dev flag
  println!("env var {}: {}", var_name, var_str);

  var_str
}
