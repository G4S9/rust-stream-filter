use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;

use crate::libs::router::HandlerFn;

pub fn factory() -> HandlerFn {
    Box::new(move |_, _| {
        Box::pin(async move {
            let mut headers = HeaderMap::new();
            headers.append("Content-Type", "application/json".parse().unwrap());

            Ok(ApiGatewayProxyResponse {
                status_code: 404,
                headers,
                multi_value_headers: Default::default(),
                body: Some(Body::from("{}")),
                is_base64_encoded: false,
            })
        })
    })
}
