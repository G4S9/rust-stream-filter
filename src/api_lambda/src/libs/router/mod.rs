use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type HandlerFn = Box<
    dyn Fn(
            ApiGatewayProxyRequest,
            Option<Box<dyn Any>>,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<ApiGatewayProxyResponse>>>>
        + Send
        + Sync
        + 'static,
>;

pub struct Router {
    default_route: Arc<HandlerFn>,
    routes: HashMap<String, Arc<HandlerFn>>,
}

impl Router {
    fn create_key(path: &str, method: &str) -> String {
        format!("{path}-{method}")
    }

    pub fn new(default_route: Arc<HandlerFn>) -> Self {
        Router {
            default_route,
            routes: HashMap::new(),
        }
    }

    pub fn set(&mut self, path: &str, method: &str, target: Arc<HandlerFn>) {
        self.routes.insert(Self::create_key(path, method), target);
    }

    pub fn get(&self, path: &str, method: &str) -> &Arc<HandlerFn> {
        self.routes
            .get(&Self::create_key(path, method))
            .unwrap_or(&self.default_route)
    }
}

#[cfg(test)]
mod tests;
