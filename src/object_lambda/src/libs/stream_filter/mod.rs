use std::mem;
use std::str::from_utf8;
use std::sync::Arc;

use bytes::{BufMut, Bytes, BytesMut};
use futures::StreamExt;
use futures_core::Stream;
use regex::Regex;
use reqwest::Error;

type BoxedSendSyncUnpinStream<I> = Box<dyn Stream<Item = I> + Send + Sync + Unpin>;

pub trait StreamFilter {
    type Item;
    fn filter_stream(
        &self,
        s: BoxedSendSyncUnpinStream<Self::Item>,
    ) -> BoxedSendSyncUnpinStream<Self::Item>;
}

#[cfg_attr(test, faux::create)]
pub struct RegexStreamFilter {
    regex: Arc<Regex>,
}

#[cfg_attr(test, faux::methods)]
impl RegexStreamFilter {
    pub fn new(regex: Regex) -> Self {
        Self {
            regex: Arc::new(regex),
        }
    }
}

#[cfg_attr(test, faux::methods)]
impl StreamFilter for RegexStreamFilter {
    type Item = Result<Bytes, Error>;
    fn filter_stream(
        &self,
        s: BoxedSendSyncUnpinStream<<Self as StreamFilter>::Item>,
    ) -> BoxedSendSyncUnpinStream<<Self as StreamFilter>::Item> {
        let mut leftover = BytesMut::new();
        let regex = self.regex.clone();
        Box::new(s.map(move |item| {
            match item {
                Ok(bytes) => {
                    let mut output = BytesMut::new();
                    let mut new_leftover = BytesMut::new();
                    mem::swap(&mut leftover, &mut new_leftover);
                    new_leftover.put(bytes);
                    let input = new_leftover.freeze();
                    let mut lines: Vec<_> = input
                        .split(|&v| v == b'\n')
                        .map(|c| from_utf8(c).unwrap_or(""))
                        .collect();
                    let last = lines.pop().unwrap_or("");
                    lines.iter().for_each(|&l| {
                        if regex.is_match(l) {
                            output.put_slice(l.as_bytes());
                            output.put_u8(b'\n');
                        }
                    });
                    // the last element might be a partial
                    if regex.is_match(last) {
                        output.put_slice(last.as_bytes());
                        output.put_u8(b'\n');
                    } else {
                        let mut new_leftover = BytesMut::from(last);
                        mem::swap(&mut leftover, &mut new_leftover);
                    }
                    Ok(output.freeze())
                }
                Err(error) => Err(error),
            }
        }))
    }
}
