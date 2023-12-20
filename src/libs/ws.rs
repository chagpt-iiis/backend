use core::str::Utf8Chunks;

use actix::{Actor, ActorContext, Addr, AsyncContext, SpawnHandle, StreamHandler};
use actix_http::ws::Item;
use actix_web::web::Bytes;
use actix_web_actors::ws;

use crate::libs::constants::{PING_INTERVAL, PING_TIMEOUT};

pub trait AppWsActor: Sized + Unpin + 'static {
    fn started(&mut self, ctx: &mut ws::WebsocketContext<WsActor<Self>>, hash: u64);
    fn stopped(&mut self, ctx: &mut ws::WebsocketContext<WsActor<Self>>, hash: u64);
    fn handle_text(&mut self, ctx: &mut ws::WebsocketContext<WsActor<Self>>, text: &str);
    fn handle_binary(&mut self, ctx: &mut ws::WebsocketContext<WsActor<Self>>, bin: Bytes);
}

pub struct WsActor<A>
where
    A: AppWsActor,
{
    pub app: A,
    tmp_buffer: Vec<u8>,
    is_tmp_buffer_string: bool,
    with_engine_io: bool,
    ping_handle: Option<SpawnHandle>,
    timeout_handle: Option<SpawnHandle>,
}

impl<A> WsActor<A>
where
    A: AppWsActor,
{
    pub const fn new(app: A, withEngineIO: bool) -> Self {
        Self {
            app,
            tmp_buffer: Vec::new(),
            is_tmp_buffer_string: false,
            with_engine_io: withEngineIO,
            ping_handle: None,
            timeout_handle: None,
        }
    }

    fn refresh_ping(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        fn ping_scheduler<A: AppWsActor>(
            _actor: &mut WsActor<A>,
            ctx: &mut ws::WebsocketContext<WsActor<A>>,
        ) {
            ctx.text("2");
        }

        if let Some(handle) = self.ping_handle.take() {
            ctx.cancel_future(handle);
        }
        self.ping_handle = Some(ctx.run_later(PING_INTERVAL, ping_scheduler));
    }

    fn refresh_timeout(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        fn close_scheduler<A: AppWsActor>(
            _actor: &mut WsActor<A>,
            ctx: &mut ws::WebsocketContext<WsActor<A>>,
        ) {
            ctx.stop();
        }

        if let Some(handle) = self.timeout_handle.take() {
            ctx.cancel_future(handle);
        }
        self.timeout_handle = Some(ctx.run_later(PING_TIMEOUT, close_scheduler));
    }
}

impl<A> Actor for WsActor<A>
where
    A: AppWsActor,
{
    type Context = ws::WebsocketContext<Self>;

    #[inline]
    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.app.started(ctx, hack::get(&addr));
        if self.with_engine_io {
            ctx.text(r#"0{"pingInterval":18320,"pingTimeout":10240,"upgrades":[]}"#);
            self.refresh_ping(ctx);
            self.refresh_timeout(ctx);
        }
    }

    #[inline]
    fn stopped(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.app.stopped(ctx, hack::get(&addr));
    }
}

impl<A> StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsActor<A>
where
    A: AppWsActor,
{
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        const LENGTH_LIMIT: usize = 0x10_0000;
        if self.with_engine_io {
            self.refresh_timeout(ctx);
        }
        match msg {
            Ok(ws::Message::Continuation(item)) => match item {
                Item::FirstText(mut bin) => {
                    bin.truncate(LENGTH_LIMIT);
                    self.tmp_buffer = bin.into();
                    self.is_tmp_buffer_string = true;
                }
                Item::FirstBinary(mut bin) => {
                    bin.truncate(LENGTH_LIMIT);
                    self.tmp_buffer = bin.into();
                    self.is_tmp_buffer_string = false;
                }
                Item::Continue(bin) => {
                    let r = LENGTH_LIMIT - self.tmp_buffer.len();
                    let b: &[u8] = &bin;
                    self.tmp_buffer.extend_from_slice(&b[..b.len().min(r)]);
                }
                Item::Last(bin) => {
                    let r = LENGTH_LIMIT - self.tmp_buffer.len();
                    let b: &[u8] = &bin;
                    self.tmp_buffer.extend_from_slice(&b[..b.len().min(r)]);
                    let v = core::mem::take(&mut self.tmp_buffer);
                    if self.is_tmp_buffer_string {
                        self.app.handle_text(
                            ctx,
                            Utf8Chunks::new(&v).next().map_or("", |chunk| chunk.valid()),
                        );
                    } else {
                        self.app.handle_binary(ctx, v.into());
                    }
                }
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(text)) => {
                if self.with_engine_io {
                    match text.as_bytes().first().copied() {
                        Some(b'2') => {
                            self.refresh_ping(ctx);
                            ctx.text("3");
                        }
                        Some(b'3') => self.refresh_ping(ctx),
                        Some(b'4') => self.app.handle_text(
                            ctx,
                            // SAFETY: the first byte is '4', so the length must be at least 1.
                            unsafe { text.get_unchecked(1..) },
                        ),
                        _ => (),
                    }
                } else {
                    self.app.handle_text(ctx, &text);
                }
            }
            Ok(ws::Message::Binary(bin)) => self.app.handle_binary(ctx, bin),
            Err(_) => ctx.stop(),
            _ => (),
        }
    }
}

pub mod hack {
    use super::{Actor, Addr};

    struct DeadHasher(u64);

    impl std::hash::Hasher for DeadHasher {
        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            if let Ok(x) = bytes.try_into() {
                self.0 = u64::from_ne_bytes(x);
            }
        }

        #[inline]
        fn write_usize(&mut self, i: usize) {
            self.0 = i as u64;
        }
    }

    #[inline]
    pub fn get<T: Actor>(addr: &Addr<T>) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut h = DeadHasher(0);
        addr.hash(&mut h);
        h.finish()
    }
}
