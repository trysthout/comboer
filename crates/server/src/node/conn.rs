use crate::{Error, ProcessContext, Request};
use byteorder::{BigEndian, ByteOrder};
use futures_util::StreamExt;
use motore::Service;
use pin_project::pin_project;
use proto::{CtrlMsg, Encoder, Len};
#[cfg(feature = "rustls")]
use rustls::pki_types::ServerName;
use std::fmt::Debug;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[pin_project(project = ConnStreamProj)]
pub enum ConnStream {
    Tcp(#[pin] TcpStream),
    #[cfg(feature = "rustls")]
    Rustls(#[pin] tokio_rustls::TlsStream<TcpStream>),
    #[cfg(feature = "native-tls")]
    Nativetls(#[pin] tokio_native_tls::TlsStream<TcpStream>),
}

impl AsyncRead for ConnStream {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), io::Error>> {
        match self.project() {
            ConnStreamProj::Tcp(stream) => stream.poll_read(cx, buf),
            #[cfg(feature = "rustls")]
            ConnStreamProj::Rustls(stream) => stream.poll_read(cx, buf),
            #[cfg(feature = "native-tls")]
            ConnStreamProj::Nativetls(stream) => stream.poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ConnStream {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            ConnStreamProj::Tcp(stream) => stream.poll_write(cx, buf),
            #[cfg(feature = "rustls")]
            ConnStreamProj::Rustls(stream) => stream.poll_write(cx, buf),
            #[cfg(feature = "native-tls")]
            ConnStreamProj::Nativetls(stream) => stream.poll_write(cx, buf),
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        match self.project() {
            ConnStreamProj::Tcp(stream) => stream.poll_flush(cx),
            #[cfg(feature = "rustls")]
            ConnStreamProj::Rustls(stream) => stream.poll_flush(cx),
            #[cfg(feature = "native-tls")]
            ConnStreamProj::Nativetls(stream) => stream.poll_flush(cx),
        }
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        match self.project() {
            ConnStreamProj::Tcp(stream) => stream.poll_shutdown(cx),
            #[cfg(feature = "rustls")]
            ConnStreamProj::Rustls(stream) => stream.poll_shutdown(cx),
            #[cfg(feature = "native-tls")]
            ConnStreamProj::Nativetls(stream) => stream.poll_shutdown(cx),
        }
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            ConnStreamProj::Tcp(stream) => stream.poll_write_vectored(cx, bufs),
            #[cfg(feature = "rustls")]
            ConnStreamProj::Rustls(stream) => stream.poll_write_vectored(cx, bufs),
            #[cfg(feature = "native-tls")]
            ConnStreamProj::Nativetls(stream) => stream.poll_write_vectored(cx, bufs),
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        match self {
            Self::Tcp(stream) => stream.is_write_vectored(),
            #[cfg(feature = "rustls")]
            Self::Rustls(stream) => stream.is_write_vectored(),
            #[cfg(feature = "native-tls")]
            Self::Nativetls(stream) => stream.is_write_vectored(),
        }
    }
}

#[derive(Clone)]
pub enum TlsConnector {
    #[cfg(feature = "rustls")]
    Rustls(tokio_rustls::TlsConnector),

    #[cfg(feature = "native-tls")]
    Nativetls(tokio_native_tls::TlsConnector),
}

impl TlsConnector {
    #[cfg(feature = "tls")]
    pub async fn connect(&self, domain: &str, stream: TcpStream) -> Result<ConnStream, Error> {
        match self {
            #[cfg(feature = "rustls")]
            Self::Rustls(conn) => conn
                .connect(
                    ServerName::try_from(domain.to_owned())
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
                    stream,
                )
                .await
                .map_err(|e| Error::from(e))
                .map(tokio_rustls::TlsStream::Client)
                .map(ConnStream::Rustls),
            #[cfg(feature = "native-tls")]
            Self::Nativetls(conn) => conn
                .connect(domain, stream)
                .await
                .map(ConnStream::Nativetls)
                .map_err(|e| Error::from(io::Error::new(io::ErrorKind::ConnectionRefused, e))),
        }
    }
}

#[derive(Clone)]
pub enum TlsAcceptor {
    #[cfg(feature = "rustls")]
    Rustls(tokio_rustls::TlsAcceptor),

    #[cfg(feature = "native-tls")]
    Nativetls(tokio_native_tls::TlsAcceptor),
}

impl TlsAcceptor {
    #[cfg(feature = "tls")]
    pub async fn accept(&self, stream: TcpStream) -> Result<ConnStream, Error> {
        match self {
            #[cfg(feature = "rustls")]
            Self::Rustls(acceptor) => acceptor
                .accept(stream)
                .await
                .map_err(Error::from)
                .map(tokio_rustls::TlsStream::Server)
                .map(ConnStream::Rustls),
            #[cfg(feature = "native-tls")]
            Self::Nativetls(acceptor) => acceptor
                .accept(stream)
                .await
                .map(ConnStream::Nativetls)
                .map_err(|e| Error::from(io::Error::new(io::ErrorKind::ConnectionAborted, e))),
        }
    }
}

#[pin_project]
#[derive(Debug)]
pub struct Connection<T, C> {
    conn: Option<T>,
    buf: Vec<u8>,
    cx: ProcessContext<C>,
    tx: UnboundedSender<Vec<u8>>,
    rx: Option<UnboundedReceiver<Vec<u8>>>,
}

impl<T, C> Connection<T, C>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
    C: Clone + Send + Sync + Debug,
{
    pub fn new(conn: T, cx: ProcessContext<C>) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            conn: Some(conn),
            buf: vec![0; 1024],
            cx,
            tx,
            rx: Some(rx),
        }
    }

    pub fn get_cx(&mut self) -> &mut ProcessContext<C> {
        &mut self.cx
    }

    pub fn get_sender(&self) -> UnboundedSender<Vec<u8>> {
        self.tx.clone()
    }

    pub async fn serving(&mut self) -> Result<(), Error> {
        let mut rx = self.rx.take().unwrap();
        loop {
            tokio::select! {
                Some(data) = rx.recv() => {
                    self.conn.as_mut().unwrap().write_all(&data).await?;
                },
                res = self.receive_data() => {
                    let length = res?;
                    if length == 0 {
                        continue;
                    }

                    let req = Request::from_msg(self.buf[..length].to_vec());
                    let dispatcher = self.cx.get_dispatcher();
                    let mut resp = dispatcher.call(&mut self.cx, req).await?.into_msg();

                    while let Some(data) = resp.next().await {
                        let data = data.map_err(Into::<Error>::into)?;
                        self.conn.as_mut().unwrap().write_all(&data).await?;
                    }
                }
            }
        }
    }

    pub async fn send_req_data<T1, T2>(&mut self, req: CtrlMsg<T1, T2>) -> Result<(), Error>
    where
        T1: Encoder<Error = anyhow::Error> + Len + Sync + Send,
        T2: Encoder<Error = anyhow::Error> + Len + Sync + Send,
    {
        let mut buf = Vec::with_capacity(4 + req.len());
        req.encode(&mut buf)?;
        self.conn.as_mut().unwrap().write_all(&buf).await?;
        Ok(())
    }

    async fn receive_data(&mut self) -> Result<usize, Error> {
        if let Err(err) = self
            .conn
            .as_mut()
            .unwrap()
            .read_exact(&mut self.buf[..4])
            .await
        {
            return Err(err.into());
        }

        let length = BigEndian::read_u32(&self.buf[..4]) as usize;
        // Erlang Tick
        if length == 0 {
            self.conn.as_mut().unwrap().write_all(&[0, 0, 0, 0]).await?;
            return Ok(length);
        }

        if length > self.buf.len() {
            self.buf.resize(length, 0)
        }

        if let Err(err) = self
            .conn
            .as_mut()
            .unwrap()
            .read_exact(&mut self.buf[..length])
            .await
        {
            return Err(err.into());
        }

        Ok(length)
    }
}

impl From<TcpStream> for ConnStream {
    #[inline]
    fn from(s: TcpStream) -> Self {
        Self::Tcp(s)
    }
}

#[cfg(feature = "rustls")]
impl From<tokio_rustls::TlsStream<TcpStream>> for ConnStream {
    #[inline]
    fn from(s: tokio_rustls::TlsStream<TcpStream>) -> Self {
        Self::Rustls(s)
    }
}

#[cfg(feature = "native-tls")]
impl From<tokio_native_tls::TlsStream<TcpStream>> for ConnStream {
    #[inline]
    fn from(s: tokio_native_tls::TlsStream<TcpStream>) -> Self {
        Self::Nativetls(s)
    }
}
