extern crate futures;

// Originally from
// https://github.com/alexcrichton/futures-rs/blob/master/tests/support/mod.rs
//
// Copyright (c) 2016 Alex Crichton
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use futures::{Async, Future, Stream};
use futures::executor::{self, Notify, NotifyHandle};
use std::fmt;
use std::sync::Arc;

pub fn notify_noop() -> NotifyHandle {
    struct Noop;
    impl Notify for Noop {
        fn notify(&self, _id: usize) {}
    }
    const NOOP: &'static Noop = &Noop;
    NotifyHandle::from(NOOP)
}

pub fn notify_panic() -> NotifyHandle {
    struct Panic;
    impl Notify for Panic {
        fn notify(&self, _id: usize) {
            panic!("should not be notified");
        }
    }
    NotifyHandle::from(Arc::new(Panic))
}

pub fn assert_done<T, F>(f: F, result: Result<T::Item, T::Error>)
where
    T: Future,
    T::Item: Eq + fmt::Debug,
    T::Error: Eq + fmt::Debug,
    F: FnOnce() -> T,
{
    assert_eq!(f().wait(), result);
}

pub fn assert_empty<T: Future, F: FnMut() -> T>(mut f: F) {
    assert!(
        executor::spawn(f())
            .poll_future_notify(&notify_panic(), 0)
            .ok()
            .unwrap()
            .is_not_ready()
    );
}

pub fn sassert_done<S: Stream>(s: &mut S) {
    match executor::spawn(s).poll_stream_notify(&notify_panic(), 0) {
        Ok(Async::Ready(None)) => {}
        Ok(Async::Ready(Some(_))) => panic!("stream had more elements"),
        Ok(Async::NotReady) => panic!("stream wasn't ready"),
        Err(_) => panic!("stream had an error"),
    }
}

pub fn sassert_empty<S: Stream>(s: &mut S) {
    match executor::spawn(s).poll_stream_notify(&notify_noop(), 0) {
        Ok(Async::Ready(None)) => panic!("stream is at its end"),
        Ok(Async::Ready(Some(_))) => panic!("stream had more elements"),
        Ok(Async::NotReady) => {}
        Err(_) => panic!("stream had an error"),
    }
}

pub fn sassert_next<S: Stream>(s: &mut S, item: S::Item)
where
    S::Item: Eq + fmt::Debug,
{
    match executor::spawn(s).poll_stream_notify(&notify_panic(), 0) {
        Ok(Async::Ready(None)) => panic!("stream is at its end"),
        Ok(Async::Ready(Some(e))) => assert_eq!(e, item),
        Ok(Async::NotReady) => panic!("stream wasn't ready"),
        Err(_) => panic!("stream had an error"),
    }
}

pub fn sassert_err<S: Stream>(s: &mut S, err: S::Error)
where
    S::Error: Eq + fmt::Debug,
{
    match executor::spawn(s).poll_stream_notify(&notify_panic(), 0) {
        Ok(Async::Ready(None)) => panic!("stream is at its end"),
        Ok(Async::Ready(Some(_))) => panic!("stream had more elements"),
        Ok(Async::NotReady) => panic!("stream wasn't ready"),
        Err(e) => assert_eq!(e, err),
    }
}
