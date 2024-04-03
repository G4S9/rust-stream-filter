use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};

use super::*;

fn get_test_request() -> ApiGatewayProxyRequest {
    ApiGatewayProxyRequest {
        ..Default::default()
    }
}

fn get_handler(status_code: i64) -> HandlerFn {
    Box::new(move |_, _| {
        Box::pin(async move {
            Ok(ApiGatewayProxyResponse {
                status_code,
                ..Default::default()
            })
        })
    })
}

#[tokio::test]
async fn test_router_new() {
    // given
    let default_handler = Arc::new(get_handler(404));
    let router = Router::new(default_handler);

    // when
    let route = router.get("/non-existing", "anyMethod");

    // then
    let result = route(get_test_request(), None).await;
    assert!(result.is_ok());
    assert_eq!(router.routes.len(), 0);
    assert_eq!(result.unwrap().status_code, 404);
}

#[tokio::test]
async fn test_router_set_and_get() {
    // given
    let default_handler = Arc::new(get_handler(404));
    let mut router = Router::new(default_handler);
    let path = "/test";
    let method = "GET";
    let handler = Arc::new(get_handler(200));
    router.set(path, method, handler);

    // when
    let route = router.get(path, method);

    // then
    let result = route(get_test_request(), None).await;
    assert!(result.is_ok());
    assert_eq!(router.routes.len(), 1);
    assert_eq!(result.unwrap().status_code, 200);
}
