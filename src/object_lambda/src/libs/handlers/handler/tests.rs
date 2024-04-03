use crate::libs::stream_byte_stream_adapter::StreamByteStreamAdapter;
use aws_lambda_events::http;
use aws_lambda_events::s3::object_lambda::GetObjectContext;
use aws_sdk_s3::operation::write_get_object_response::WriteGetObjectResponseOutput;
use aws_sdk_s3::primitives::ByteStream;

use super::*;

#[tokio::test]
async fn test_happy_path() {
    // given
    let mut mock_s3 = s3::S3::faux();
    faux::when!(mock_s3.write_get_object_response)
        .then(|(_, _, _)| Ok(WriteGetObjectResponseOutput::builder().build()));

    let mut mock_reqwest = reqwest::Reqwest::faux();
    faux::when!(mock_reqwest.get).then(|_| {
        Ok(http::Response::builder()
            .status(200)
            .body("")
            .unwrap()
            .into())
    });

    let mut mock_stream_filter = RegexStreamFilter::faux();
    faux::when!(mock_stream_filter.filter_stream).then(|_| {
        Box::new(
            ::reqwest::Response::from(http::Response::builder().status(200).body("").unwrap())
                .bytes_stream(),
        )
    });

    let mut mock_stream_byte_stream_adapter = StreamByteStreamAdapter::faux();
    faux::when!(mock_stream_byte_stream_adapter.stream_to_byte_stream)
        .then(|_| ByteStream::from_static(b""));

    let handler = factory(
        Arc::new(mock_s3),
        Arc::new(mock_reqwest),
        Arc::new(mock_stream_filter),
        Arc::new(mock_stream_byte_stream_adapter),
    );

    let event = S3ObjectLambdaEvent {
        get_object_context: Some(GetObjectContext {
            input_s3_url: "https://example.com".to_string(),
            output_route: "output_route".to_string(),
            output_token: "output_token".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };

    // when
    let response = handler(event).await;

    // then
    assert_eq!(response.unwrap().status_code, 200);
    // ... and so on
}

#[tokio::test]
async fn and_so_on() {
    // ...
}
