use aws_sdk_s3 as s3;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::delete_object::{DeleteObjectError, DeleteObjectOutput};
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::head_object::{HeadObjectError, HeadObjectOutput};
use aws_sdk_s3::operation::list_objects_v2::{ListObjectsV2Error, ListObjectsV2Output};
use aws_sdk_s3::operation::put_object::PutObjectError;
use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};

#[cfg_attr(test, faux::create)]
pub struct S3 {
    inner: s3::Client,
}

#[cfg_attr(test, faux::methods)]
impl S3 {
    pub fn new(inner: s3::Client) -> Self {
        Self { inner }
    }

    pub async fn head_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<HeadObjectOutput, SdkError<HeadObjectError>> {
        self.inner
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
    }

    pub async fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<DeleteObjectOutput, SdkError<DeleteObjectError>> {
        self.inner
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
    }

    pub async fn list_objects_v2(
        &self,
        bucket: &str,
        continuation_token: Option<&str>,
        max_keys: Option<i32>,
    ) -> Result<ListObjectsV2Output, SdkError<ListObjectsV2Error>> {
        let mut builder = self.inner.list_objects_v2().bucket(bucket);

        if let Some(continuation_token) = continuation_token {
            builder = builder.continuation_token(continuation_token);
        }

        if let Some(max_keys) = max_keys {
            builder = builder.max_keys(max_keys);
        }

        builder.send().await
    }

    pub async fn get_object_presigned(
        &self,
        bucket: &str,
        key: &str,
        presigning_config: PresigningConfig,
        response_content_type: Option<&str>,
        response_content_disposition: Option<&str>,
    ) -> Result<PresignedRequest, SdkError<GetObjectError>> {
        let mut builder = self.inner.get_object().bucket(bucket).key(key);

        if let Some(response_content_type) = response_content_type {
            builder = builder.response_content_type(response_content_type);
        }

        if let Some(response_content_disposition) = response_content_disposition {
            builder = builder.response_content_disposition(response_content_disposition);
        }

        builder.presigned(presigning_config).await
    }

    pub async fn put_object_presigned(
        &self,
        bucket: &str,
        key: &str,
        presigning_config: PresigningConfig,
        content_type: Option<&str>,
        content_disposition: Option<&str>,
    ) -> Result<PresignedRequest, SdkError<PutObjectError>> {
        let mut builder = self.inner.put_object().bucket(bucket).key(key);

        if let Some(content_type) = content_type {
            builder = builder.content_type(content_type);
        }

        if let Some(content_disposition) = content_disposition {
            builder = builder.content_disposition(content_disposition);
        }

        builder.presigned(presigning_config).await
    }
}
