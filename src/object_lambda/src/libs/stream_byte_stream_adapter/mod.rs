use std::pin::Pin;
use std::task::Poll;

use aws_sdk_s3::primitives::ByteStream;
use bytes::Bytes;
use futures_core::Stream;
use http_body::Frame;
use reqwest::Error;

type BoxedSendSyncUnpinStream<I> = Box<dyn Stream<Item = I> + Send + Sync + Unpin>;

pub trait StreamToByteStream {
    type Item;
    fn stream_to_byte_stream(&self, stream: BoxedSendSyncUnpinStream<Self::Item>) -> ByteStream;
}

#[cfg_attr(test, faux::create)]
pub struct StreamByteStreamAdapter;

#[cfg_attr(test, faux::methods)]
impl StreamByteStreamAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[cfg_attr(test, faux::methods)]
impl StreamToByteStream for StreamByteStreamAdapter {
    type Item = Result<Bytes, Error>;

    fn stream_to_byte_stream(
        &self,
        stream: BoxedSendSyncUnpinStream<<Self as StreamToByteStream>::Item>,
    ) -> ByteStream {
        StreamBodyAdapter::from(stream).into()
    }
}

impl From<StreamBodyAdapter<Result<Bytes, Error>>> for ByteStream {
    fn from(sba: StreamBodyAdapter<Result<Bytes, Error>>) -> Self {
        ByteStream::from_body_1_x(sba)
    }
}

pub struct StreamBodyAdapter<I> {
    stream: BoxedSendSyncUnpinStream<I>,
}

impl<I> From<BoxedSendSyncUnpinStream<I>> for StreamBodyAdapter<I> {
    fn from(stream: BoxedSendSyncUnpinStream<I>) -> Self {
        Self { stream }
    }
}

impl http_body::Body for StreamBodyAdapter<Result<Bytes, Error>> {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        #[allow(unused_mut)] mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Error>>> {
        let stream = Pin::new(&mut self.stream);
        match stream.poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(Frame::data(bytes)))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
