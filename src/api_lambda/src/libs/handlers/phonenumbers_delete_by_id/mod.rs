use crate::libs::deps::env;
use crate::libs::deps::s3;
use crate::libs::router::HandlerFn;
use anyhow::{anyhow, Context};
use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use clone_all::clone_all;
use std::sync::Arc;

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

            match s3.head_object(&bucket, task_id).await {
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
                _ => (),
            }

            s3.delete_object(&bucket, task_id)
                .await
                .context(format!("Error deleting object with task_id {:?}", task_id))?;

            let mut headers = HeaderMap::new();
            headers.append("Content-Type", "application/json".parse().unwrap());

            Ok(ApiGatewayProxyResponse {
                status_code: 200,
                headers,
                multi_value_headers: Default::default(),
                body: Some(Body::from("{}")),
                is_base64_encoded: false,
            })
        })
    })
}

#[cfg(test)]
mod tests;
