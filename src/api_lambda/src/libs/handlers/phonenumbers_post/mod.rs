use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::libs::deps::{env, s3};
use anyhow::Context;
use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use aws_sdk_s3::presigning::PresigningConfig;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use clone_all::clone_all;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::libs::router::HandlerFn;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PhonenumbersPostRequestBody {
    file_name: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PhonenumbersPostResponseBody {
    task_id: String,
    url: String,
    signed_headers: HashMap<String, String>,
}

pub fn factory(s3: Arc<s3::S3>, env: Arc<env::Env>) -> HandlerFn {
    Box::new(move |event, _| {
        clone_all!(s3, env);
        Box::pin(async move {
            let body: String = if event.is_base64_encoded {
                String::from_utf8(
                    STANDARD
                        .decode(event.body.unwrap_or("".into()))
                        .context("Could not base64 decode event body")?,
                )
                .context("Could not UTF-8 decode event body")?
            } else {
                event.body.unwrap_or("".into())
            };

            let body: PhonenumbersPostRequestBody =
                serde_json::from_str(&body).context("Could not deserialize the message body")?;

            let task_id = Uuid::new_v4().to_string();
            let bucket = env.var("BUCKET_NAME").context("BUCKET_NAME not defined")?;
            let presigning_config = PresigningConfig::builder()
                .expires_in(Duration::from_secs(3600))
                .build()?;
            let content_type = "text/plain".to_owned();
            let content_disposition = format!("attachment; filename=\"{}\"", body.file_name);
            let url = s3
                .put_object_presigned(
                    &bucket,
                    &task_id,
                    presigning_config,
                    Some(&content_type),
                    Some(&content_disposition),
                )
                .await
                .context("Could not generate the presigning request")?
                .uri()
                .to_string();

            let mut signed_headers = HashMap::new();
            signed_headers.insert("Content-Type".into(), content_type);
            signed_headers.insert("Content-Disposition".into(), content_disposition);

            let response_body = PhonenumbersPostResponseBody {
                task_id,
                url,
                signed_headers,
            };

            let mut headers = HeaderMap::new();
            headers.append("Content-Type", "application/json".parse().unwrap());

            Ok(ApiGatewayProxyResponse {
                status_code: 200,
                headers,
                multi_value_headers: Default::default(),
                body: Some(Body::from(serde_json::to_string(&response_body)?)),
                is_base64_encoded: false,
            })
        })
    })
}
