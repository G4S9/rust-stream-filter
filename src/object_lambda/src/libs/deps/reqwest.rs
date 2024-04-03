use reqwest::{Response, Result};

#[cfg_attr(test, faux::create)]
pub struct Reqwest {}

#[cfg_attr(test, faux::methods)]
impl Reqwest {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn get(&self, url: &str) -> Result<Response> {
        reqwest::get(url).await
    }
}
