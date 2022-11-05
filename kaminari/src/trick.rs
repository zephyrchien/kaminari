use core::ops::Deref;
use std::io::Result;
use std::future::Future;

use super::{IOStream, AsyncAccept, AsyncConnect};

// Safety:
// pointer is not null once inited(comes from an immutable ref)
// pointee memory is always valid during the eventloop
pub struct Ref<T>(*const T);

unsafe impl<T: Send + Sync> Send for Ref<T> {}
unsafe impl<T: Send + Sync> Sync for Ref<T> {}

impl<T> Copy for Ref<T> {}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target { unsafe { &*self.0 } }
}

impl<T> AsRef<T> for Ref<T> {
    #[inline]
    fn as_ref(&self) -> &T { unsafe { &*self.0 } }
}

impl<T> From<&T> for Ref<T> {
    #[inline]
    fn from(x: &T) -> Self { Ref(x as *const _) }
}

impl<T> Ref<T> {
    #[inline]
    pub const fn new(x: &T) -> Self { Self(x as *const _) }
}

impl<S, T> AsyncConnect<S> for Ref<T>
where
    S: IOStream,
    T: AsyncConnect<S>,
{
    type Stream = T::Stream;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self: 'a;

    fn connect<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::ConnectFut<'a> {
        async move {
            let stream = self.as_ref().connect(stream, buf).await?;
            Ok(stream)
        }
    }
}

impl<S, T> AsyncAccept<S> for Ref<T>
where
    S: IOStream,
    T: AsyncAccept<S>,
{
    type Stream = T::Stream;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn accept<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::AcceptFut<'a> {
        async move {
            let stream = self.as_ref().accept(stream, buf).await?;
            Ok(stream)
        }
    }
}
