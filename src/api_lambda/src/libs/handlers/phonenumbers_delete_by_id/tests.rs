use std::collections::HashMap;

use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use aws_sdk_s3::operation::delete_object::DeleteObjectOutput;
use aws_sdk_s3::operation::head_object::HeadObjectOutput;

use super::*;

fn mock_handler_fn() -> HandlerFn {
    Box::new(|_, _| Box::pin(async { unimplemented!() }))
}

const ANY_BUCKET_NAME: &str = "ANY_BUCKET_NAME";
const ANY_KEY: &str = "ANY_KEY";

#[tokio::test]
async fn test_happy_path() {
    // given
    let bad_request = Arc::new(mock_handler_fn());
    let not_found = Arc::new(mock_handler_fn());
    let mut s3 = s3::S3::faux();
    let mut env = env::Env::faux();

    faux::when!(env.var(_)).then_return(Ok(ANY_BUCKET_NAME.into()));

    faux::when!(s3.head_object(_, _)).then(|(_, _)| Ok(HeadObjectOutput::builder().build()));

    faux::when!(s3.delete_object(_, _)).then(|(_, _)| Ok(DeleteObjectOutput::builder().build()));

    let handler = factory(bad_request, not_found, Arc::new(s3), Arc::new(env));
    let mut path_parameters = HashMap::new();
    path_parameters.insert(String::from("id"), String::from(ANY_KEY));

    let event = ApiGatewayProxyRequest {
        path_parameters,
        ..Default::default()
    };

    // when
    let result = handler(event, None).await;

    // then
    assert_eq!(result.unwrap().status_code, 200);
    // ... and so on
}

#[tokio::test]
async fn and_so_on() {
    // ...
}
