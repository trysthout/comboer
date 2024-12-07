use std::marker::PhantomData;
use std::sync::Arc;
use std::{
    fmt::Debug,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use dashmap::DashMap;
use futures_util::StreamExt;
use motore::{BoxCloneService, Service};

use proto::{term, CtrlMsg, Decoder, Encoder, Len};

use crate::{BoxStream, Error, RawMsg, Request, Response};

#[derive(Clone, Debug)]
pub struct Process {
    pid: term::NewPid,
    // sender: UnboundedSender<CtrlMsg>,
}

impl Process {
    pub fn new(pid: term::NewPid) -> Self {
        Self { pid }
    }

    pub fn get_pid(&self) -> term::NewPid {
        self.pid.clone()
    }

    pub fn get_pid_ref(&self) -> &term::NewPid {
        &self.pid
    }

    // pub fn send(&self, msg: CtrlMsg) {
    //    let _ = self.sender.send(msg);
    // }
}

#[derive(Debug, Clone)]
pub struct EmptyBoxCx;
static UNIQ_ID: AtomicU64 = AtomicU64::new(1);
static PID_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct ProcessContext<C> {
    node_name: String,
    creation: u32,
    curr_match_id: MatchId,
    dispatcher: Arc<Dispatcher<C, Vec<u8>>>,
    box_cx: Option<C>,
}

impl<C> ProcessContext<C>
where
    C: Debug + Clone + Send,
{
    pub(crate) fn with_dispatcher(
        node_name: String,
        creation: u32,
        dispatcher: Dispatcher<C, Vec<u8>>,
    ) -> Self {
        Self {
            node_name,
            creation,
            curr_match_id: MatchId(0),
            dispatcher: Arc::new(dispatcher),
            box_cx: None,
        }
    }

    pub(crate) fn get_dispatcher(&self) -> Arc<Dispatcher<C>> {
        self.dispatcher.clone()
    }

    pub fn add_matcher<M>(&mut self, matcher: M) -> &mut Self
    where
        M: Service<ProcessContext<C>, Request<Vec<u8>>, Response = Response<RawMsg>, Error = Error>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let matcher_id = MatchId::next();
        self.dispatcher.add_matcher(matcher_id, matcher);
        self
    }

    fn set_match_id(&mut self, id: MatchId) {
        self.curr_match_id = id;
    }

    pub fn make_process(&self) -> Process {
        let pid_id = PID_ID.fetch_add(1, Ordering::Relaxed);
        let pid = term::NewPid {
            node: term::SmallAtomUtf8(self.node_name.clone()),
            id: (pid_id & 0x7fff) as u32,
            serial: ((pid_id >> 15) & 0x1fff) as u32,
            creation: self.creation,
        };

        Process::new(pid)
    }

    /// create ref. refer to https://github.com/erlang/otp/blob/master/lib/erl_interface/src/connect/ei_connect.c#L745
    pub fn make_ref(&self) -> term::NewerReference {
        let uniq_id = UNIQ_ID.fetch_add(1, Ordering::Relaxed);
        term::NewerReference {
            length: 3,
            node: term::SmallAtomUtf8(self.node_name.clone()),
            creation: self.creation,
            id: vec![
                (uniq_id & 0x3ffff) as u32,
                ((uniq_id >> 18) & 0xffffffff) as u32,
                ((uniq_id >> (18 + 32)) & 0xffffffff) as u32,
            ],
        }
    }

    pub fn set_box_cx(&mut self, cx: C) {
        self.box_cx = Some(cx)
    }

    pub fn get_box_cx(&self) -> Option<&C> {
        self.box_cx.as_ref()
    }

    pub fn get_mut_box_cx(&mut self) -> Option<&mut C> {
        self.box_cx.as_mut()
    }
}

static MATCH_ID: AtomicU32 = AtomicU32::new(0);
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MatchId(u32);

impl MatchId {
    pub fn next() -> Self {
        let id = MATCH_ID.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }
}

type Matchers<C, T> =
    DashMap<MatchId, BoxCloneService<ProcessContext<C>, Request<T>, Response<RawMsg>, Error>>;

#[derive(Debug, Clone)]
pub(crate) struct Dispatcher<C, T = Vec<u8>> {
    matchers: Matchers<C, T>,
}

impl<C, T> Dispatcher<C, T>
where
    T: 'static,
{
    pub(crate) fn new() -> Self {
        Self {
            matchers: DashMap::new(),
        }
    }

    pub(crate) fn add_matcher<M>(&self, matcher_id: MatchId, matcher: M)
    where
        M: Service<ProcessContext<C>, Request<T>, Response = Response<RawMsg>, Error = Error>
            + Send
            + Sync
            + Clone
            + 'static,
    {
        self.matchers
            .insert(matcher_id, BoxCloneService::new(matcher));
    }
}

impl<C, T> Service<ProcessContext<C>, Request<T>> for Dispatcher<C, T>
where
    C: Debug + Clone + Send + Sync,
    T: Send + Sync + Clone,
{
    type Response = Response<RawMsg>;
    type Error = Error;
    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut ProcessContext<C>,
        req: Request<T>,
    ) -> Result<Self::Response, Self::Error> {
        for matcher in self.matchers.iter() {
            let req = req.clone();
            let resp = matcher.call(cx, req).await?;
            if !resp.get_msg().is_empty() {
                cx.set_match_id(*matcher.key());

                return Ok(resp);
            }
        }
        Ok(Response::new(RawMsg::new_empty()))
    }
}

pub struct DistCodec<S, T1, T2, U> {
    inner: S,
    _phat: PhantomData<(T1, T2, U)>,
}

impl<S, T1, T2, U> DistCodec<S, T1, T2, U> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            _phat: PhantomData,
        }
    }
}

impl<S, T1, T2, U> Clone for DistCodec<S, T1, T2, U>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phat: PhantomData,
        }
    }
}

impl<S, C, T1, T2, U> Service<ProcessContext<C>, Request<Vec<u8>>> for DistCodec<S, T1, T2, U>
where
    S: Service<
            ProcessContext<C>,
            Request<CtrlMsg<T1, T2>>,
            Response = Response<Option<BoxStream<'static, U>>>,
        > + Clone
        + Send
        + Sync
        + 'static,
    S::Error: Into<crate::Error>,
    T1: Decoder<Error = anyhow::Error> + Len + Send + Sync + Clone + Debug + 'static,
    T2: Decoder<Error = anyhow::Error> + Len + Send + Sync + Clone + Debug + 'static,
    U: Encoder<Error = anyhow::Error> + Len + Send + Sync + Debug + 'static,
    C: Send + Sync,
{
    type Response = Response<RawMsg>;
    type Error = Error;

    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut ProcessContext<C>,
        req: Request<Vec<u8>>,
    ) -> Result<Self::Response, Self::Error> {
        let msg = req.get_msg();
        let ctrl_msg = CtrlMsg::<T1, T2>::try_from(msg.as_ref())
            .map_err(Into::<Error>::into)
            .ok();
        if ctrl_msg.is_none() {
            return Ok(Response::new(RawMsg::new_empty()));
        }

        let stream = self
            .inner
            .call(cx, Request::from_msg(ctrl_msg.unwrap()))
            .await
            .map_err(Into::into)?
            .into_msg();

        if let Some(stream) = stream {
            let raw_msg = RawMsg::new(Box::pin(async_stream::stream! {
               futures_util::pin_mut!(stream);
                while let Some(msg) = stream.next().await {
                    let mut buf = Vec::with_capacity(4 + msg.len());
                    msg.encode(&mut buf)?;
                    yield Ok(buf)
                }
            }));

            return Ok(Response::new(raw_msg));
        }

        Ok(Response::new(RawMsg::new_empty()))
    }
}

#[derive(Clone)]
pub struct ServiceBuilder<S> {
    service: S,
}

impl<S> ServiceBuilder<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> ServiceBuilder<S> {
    pub fn build<C, T1, T2, U>(self) -> DistCodec<S, T1, T2, U>
    where
        S: Service<
            ProcessContext<C>,
            Request<CtrlMsg<T1, T2>>,
            Response = Response<Option<BoxStream<'static, U>>>,
            Error = Error,
        >,
    {
        let service = motore::builder::ServiceBuilder::new().service(self.service);

        DistCodec::new(service)
    }
}
