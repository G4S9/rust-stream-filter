use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use aws_lambda_events::s3::object_lambda::S3ObjectLambdaEvent;
use clone_all::clone_all;
use serde::Serialize;

use crate::libs::deps::reqwest;
use crate::libs::deps::s3;
use crate::libs::stream_byte_stream_adapter::{StreamByteStreamAdapter, StreamToByteStream};
use crate::libs::stream_filter::{RegexStreamFilter, StreamFilter};

#[derive(Serialize, Debug)]
pub struct ObjectLambdaResponse {
    status_code: u32,
}

pub type HandlerFn = Box<
    dyn Fn(
            S3ObjectLambdaEvent,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<ObjectLambdaResponse>>>>
        + Send
        + Sync
        + 'static,
>;

pub fn factory(
    s3: Arc<s3::S3>,
    reqwest: Arc<reqwest::Reqwest>,
    filter: Arc<RegexStreamFilter>,
    adapter: Arc<StreamByteStreamAdapter>,
) -> HandlerFn {
    Box::new(move |event| {
        clone_all!(s3, reqwest, filter, adapter);
        Box::pin(async move {
            tracing::info!("Received event: {:?}", event);
            let get_object_context = event
                .get_object_context
                .ok_or(anyhow!("GetObjectContext not defined"))?;
            let output_route = get_object_context.output_route;
            let output_token = get_object_context.output_token;
            let input_s3_url = get_object_context.input_s3_url;
            let stream = reqwest
                .get(&input_s3_url)
                .await
                .context("could not fetch input_s3_url")?
                .bytes_stream();

            let stream = filter.filter_stream(Box::new(stream));

            s3.write_get_object_response(
                &output_route,
                &output_token,
                adapter.stream_to_byte_stream(stream),
            )
            .await
            .context("error in writing get_object_response")?;

            Ok(ObjectLambdaResponse { status_code: 200 })
        })
    })
}

#[cfg(test)]
mod tests;
