use std::sync::Arc;

use anyhow::anyhow;
use aws_config::BehaviorVersion;
use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};
use tokio::sync::OnceCell;

use libs::deps::s3;

use crate::libs::deps::env;
use crate::libs::handlers::*;
use crate::libs::router::Router;

mod libs;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

static ROUTER: OnceCell<Router> = OnceCell::const_new();

async fn get_or_init_router() -> &'static Router {
    ROUTER
        .get_or_init(|| async {
            let s3: Arc<s3::S3> = Arc::new(
                async {
                    s3::S3::new(aws_sdk_s3::Client::new(
                        &aws_config::defaults(BehaviorVersion::latest()).load().await,
                    ))
                }
                .await,
            );
            let env = Arc::new(env::Env::new());
            let not_found = Arc::new(not_found::factory());
            let bad_request = Arc::new(bad_request::factory());
            let phonenumbers_get = Arc::new(phonenumbers_get::factory(s3.clone(), env.clone()));
            let phone_numbers_post = Arc::new(phonenumbers_post::factory(s3.clone(), env.clone()));
            let phone_numbers_get_by_id = Arc::new(phonenumbers_get_by_id::factory(
                bad_request.clone(),
                not_found.clone(),
                s3.clone(),
                env.clone(),
            ));
            let phonenumbers_delete_by_id = Arc::new(phonenumbers_delete_by_id::factory(
                bad_request.clone(),
                not_found.clone(),
                s3.clone(),
                env.clone(),
            ));
            let mut router = Router::new(not_found);
            router.set("/phonenumbers", "GET", phonenumbers_get);
            router.set("/phonenumbers", "POST", phone_numbers_post);
            router.set("/phonenumbers/{id}", "GET", phone_numbers_get_by_id);
            router.set("/phonenumbers/{id}", "DELETE", phonenumbers_delete_by_id);
            router
        })
        .await
}

async fn my_handler(
    LambdaEvent { payload: event, .. }: LambdaEvent<ApiGatewayProxyRequest>,
) -> anyhow::Result<ApiGatewayProxyResponse> {
    let router = get_or_init_router();

    tracing::info!("received event: {:?}", event);

    let resource = event
        .resource
        .as_ref()
        .ok_or(anyhow!("Resource not defined"))?;
    let method = event.http_method.as_ref();

    router.await.get(resource, method)(event, None).await
}
