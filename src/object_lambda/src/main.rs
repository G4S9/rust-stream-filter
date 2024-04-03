use std::sync::Arc;

use aws_config::BehaviorVersion;
use aws_lambda_events::s3::object_lambda::S3ObjectLambdaEvent;
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};
use regex::Regex;
use tokio::sync::OnceCell;

use crate::libs::deps::reqwest::Reqwest;
use crate::libs::deps::s3;
use crate::libs::handlers::handler::{HandlerFn, ObjectLambdaResponse};
use crate::libs::stream_byte_stream_adapter::StreamByteStreamAdapter;
use crate::libs::stream_filter::RegexStreamFilter;

mod libs;

static HANDLER: OnceCell<HandlerFn> = OnceCell::const_new();

async fn get_or_init_handler() -> &'static HandlerFn {
    HANDLER
        .get_or_init(|| async {
            let s3 = Arc::new(s3::S3::new(aws_sdk_s3::Client::new(
                &aws_config::defaults(BehaviorVersion::latest()).load().await,
            )));
            let reqwest = Arc::new(Reqwest::new());
            let filter = Arc::new(RegexStreamFilter::new(
                Regex::new(r"^\s*(\+|0\s*0)\s*3\s*6\s*(1|[2-9]\s*[0-9])\s*([0-9]\s*){7}$").unwrap(),
            ));
            let adapter = Arc::new(StreamByteStreamAdapter::new());
            libs::handlers::handler::factory(s3, reqwest, filter, adapter)
        })
        .await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn my_handler(
    LambdaEvent { payload: event, .. }: LambdaEvent<S3ObjectLambdaEvent>,
) -> anyhow::Result<ObjectLambdaResponse> {
    let handler = get_or_init_handler().await;

    tracing::info!("received event: {:?}", event);

    handler(event).await
}
