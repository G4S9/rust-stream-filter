use std::env;

#[cfg_attr(test, faux::create)]
pub struct Env {}

#[cfg_attr(test, faux::methods)]
impl Env {
    pub fn new() -> Self {
        Env {}
    }

    pub fn var(&self, key: &str) -> Result<String, env::VarError> {
        env::var(key)
    }
}
