use aws_sdk_s3 as s3;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::write_get_object_response::{
    WriteGetObjectResponseError, WriteGetObjectResponseOutput,
};
use aws_sdk_s3::primitives::ByteStream;

#[cfg_attr(test, faux::create)]
pub struct S3 {
    inner: s3::Client,
}

#[cfg_attr(test, faux::methods)]
impl S3 {
    pub fn new(inner: s3::Client) -> Self {
        Self { inner }
    }
    pub async fn write_get_object_response(
        &self,
        output_route: &str,
        output_token: &str,
        byte_stream: ByteStream,
    ) -> Result<WriteGetObjectResponseOutput, SdkError<WriteGetObjectResponseError>> {
        self.inner
            .write_get_object_response()
            .request_route(output_route)
            .request_token(output_token)
            .body(byte_stream)
            .send()
            .await
    }
}
