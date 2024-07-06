use std::fmt::Debug;

use byteorder::{BigEndian, ByteOrder};
use futures_util::StreamExt;
use motore::Service;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use proto::{CtrlMsg, Encoder, Len};

use crate::{Error, ProcessContext, Request};

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct Connection<T, C> {
        conn: Option<T>,
        buf: Vec<u8>,
        cx: ProcessContext<C>,
        tx: UnboundedSender<Vec<u8>>,
        rx: Option<UnboundedReceiver<Vec<u8>>>,
    }
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
            self.conn.as_mut().unwrap().write(&[0, 0, 0, 0]).await?;
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
