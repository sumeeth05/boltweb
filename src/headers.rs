use pin_project_lite::pin_project;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pin_project! {
    pub struct LimitReader<T> {
        #[pin]
        inner: T,
        max: usize,
        read: usize,
    }
}

impl<T: AsyncRead> LimitReader<T> {
    pub fn new(inner: T, max: usize) -> Self {
        Self {
            inner,
            max,
            read: 0,
        }
    }
}

impl<T: AsyncRead> AsyncRead for LimitReader<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();

        let before = this.read.clone();
        let poll = this.inner.poll_read(context, buf);

        if let Poll::Ready(Ok(())) = &poll {
            let new_bytes = buf.filled().len().saturating_sub(before);
            *this.read += new_bytes;

            if *this.read > *this.max {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "header size limit exceeded",
                )));
            }
        }

        poll
    }
}

impl<T: AsyncWrite> AsyncWrite for LimitReader<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_write(context, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_flush(context)
    }

    fn poll_shutdown(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_shutdown(context)
    }
}
