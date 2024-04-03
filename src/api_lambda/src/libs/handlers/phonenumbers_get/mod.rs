use crate::libs::deps::{env, s3};
use anyhow::Context;
use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use clone_all::clone_all;
use serde::Serialize;
use std::sync::Arc;

use crate::libs::router::HandlerFn;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PhonenumbersGetResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation_token: Option<String>,
    task_ids: Vec<String>,
}

impl PhonenumbersGetResponse {
    fn new() -> Self {
        Self {
            continuation_token: None,
            task_ids: vec![],
        }
    }
}

pub fn factory(s3: Arc<s3::S3>, env: Arc<env::Env>) -> HandlerFn {
    Box::new(move |event, _| {
        clone_all!(s3, env);
        Box::pin(async move {
            let bucket = env
                .var("BUCKET_NAME")
                .context("BUCKET_NAME env var not defined")?;

            let continuation_token = event.query_string_parameters.first("continuation_token");

            let list_response = s3
                .list_objects_v2(&bucket, continuation_token, Some(100))
                .await?;

            let mut phonenumbers_get_response = PhonenumbersGetResponse::new();

            if let Some(continuation_token) = list_response.next_continuation_token {
                phonenumbers_get_response.continuation_token = Some(continuation_token);
            }

            if let Some(contents) = list_response.contents {
                phonenumbers_get_response.task_ids =
                    contents.iter().filter_map(|v| v.key.clone()).collect();
            }

            let mut headers = HeaderMap::new();
            headers.append("Content-Type", "application/json".parse().unwrap());

            Ok(ApiGatewayProxyResponse {
                status_code: 200,
                headers,
                multi_value_headers: Default::default(),
                body: Some(Body::from(serde_json::to_string(
                    &phonenumbers_get_response,
                )?)),
                is_base64_encoded: false,
            })
        })
    })
}
