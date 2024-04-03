use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use serde::Serialize;

use crate::libs::router::HandlerFn;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BadRequestResponseBody {
    message: String,
}

pub fn factory() -> HandlerFn {
    Box::new(move |_, extra| {
        Box::pin(async move {
            let mut headers = HeaderMap::new();
            headers.append("Content-Type", "application/json".parse().unwrap());

            let message: String = match extra {
                Some(boxed_any) => boxed_any
                    .downcast_ref::<String>()
                    .unwrap_or(&String::from(""))
                    .clone(),
                _ => String::from(""),
            };

            Ok(ApiGatewayProxyResponse {
                status_code: 400,
                headers,
                multi_value_headers: Default::default(),
                body: Some(Body::from(serde_json::to_string(
                    &BadRequestResponseBody { message },
                )?)),
                is_base64_encoded: false,
            })
        })
    })
}
