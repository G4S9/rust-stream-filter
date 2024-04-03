use std::sync::Arc;
use std::time::Duration;

use crate::libs::deps::{env, s3};
use anyhow::{anyhow, Context};
use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use aws_sdk_s3::presigning::PresigningConfig;
use clone_all::clone_all;
use serde::Serialize;

use crate::libs::router::HandlerFn;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PhonenumbersGetByIdResponseBody {
    url: String,
}

pub fn factory(
    bad_request: Arc<HandlerFn>,
    not_found: Arc<HandlerFn>,
    s3: Arc<s3::S3>,
    env: Arc<env::Env>,
) -> HandlerFn {
    Box::new(move |event, _| {
        clone_all!(bad_request, not_found, s3, env);
        Box::pin(async move {
            let task_id = event.path_parameters.get("id");

            if task_id.is_none() {
                return bad_request(
                    event,
                    Some(Box::new(String::from("Path parameter `id` missing"))),
                )
                .await;
            }

            let task_id = task_id.unwrap();

            let bucket = env.var("BUCKET_NAME").context("BUCKET_NAME not defined")?;

            let head_object_response = match s3.head_object(&bucket, task_id).await {
                Err(sdk_error)
                    if sdk_error
                        .as_service_error()
                        .is_some_and(|e| e.is_not_found()) =>
                {
                    println!("sdk_error 1: {:?}", sdk_error);
                    return not_found(event, None).await;
                }
                Err(sdk_error) => {
                    println!("sdk_error 2: {:?}", sdk_error);
                    return Err(anyhow!(sdk_error));
                }
                Ok(response) => response,
            };

            let bucket = env
                .var("OBJECT_LAMBDA_ENDPOINT_ARN")
                .context("OBJECT_LAMBDA_ENDPOINT_ARN not defined")?;

            let presigning_config = PresigningConfig::builder()
                .expires_in(Duration::from_secs(3600))
                .build()?;

            let url = s3
                .get_object_presigned(
                    &bucket,
                    task_id,
                    presigning_config,
                    head_object_response.content_type.as_deref(),
                    head_object_response.content_disposition.as_deref(),
                )
                .await?
                .uri()
                .to_owned();

            let mut headers = HeaderMap::new();
            headers.append("Content-Type", "application/json".parse().unwrap());

            Ok(ApiGatewayProxyResponse {
                status_code: 200,
                headers,
                multi_value_headers: Default::default(),
                body: Some(Body::from(serde_json::to_string(
                    &PhonenumbersGetByIdResponseBody { url },
                )?)),
                is_base64_encoded: false,
            })
        })
    })
}
