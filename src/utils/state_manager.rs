use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys::{console};

#[derive(Clone)]
pub struct Store<S: Clone, A> {
  state: S,
  reducer: fn(&S, A) -> Result<S, JsValue>,
}

impl<S: Clone + fmt::Debug, A> fmt::Debug for Store<S, A> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "state: {:#?}", self.state)
  }
}

impl<S: Clone, A> Store<S, A> {
  pub fn init(
    reducer: fn(&S, A) -> Result<S, JsValue>,
    initial_state: S,
  ) -> Store<S, A> {
    Store {
      state: initial_state,
      reducer,
    }
  }
}

pub fn get_state(&self) -> &S {
  &self.state
}

pub fn dispatch(&mut self, action: A) {
  match (self.reducer)(&self.state, action) {
    Ok(new_state) => {
      self.state = new_state;
    },
    Err(e) => {
      console::log_2(&"Error updating state: ".into(), &e.into());
    }
  }
}