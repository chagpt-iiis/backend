use actix::{Addr, AsyncContext, Handler};
use actix_web::web::Bytes;
use actix_web_actors::ws;
use parking_lot::RwLock;

use crate::libs::{ws::{AppWsActor, WsActor}, constants::EMITTER_SECRET};

use super::danmaku::BroadcastDanmaku;

pub struct DanmakuEmitter;
pub type DanmakuEmitterWs = WsActor<DanmakuEmitter>;
pub type DanmakuEmitterContext = ws::WebsocketContext<DanmakuEmitterWs>;
type MaybeAddress = Option<Addr<DanmakuEmitterWs>>;

pub static CURRENT_EMITTER: RwLock<MaybeAddress> = RwLock::new(None);

impl AppWsActor for DanmakuEmitter {
    fn started(&mut self, _: &mut DanmakuEmitterContext, _: u64) {}

    fn stopped(&mut self, ctx: &mut DanmakuEmitterContext, _: u64) {
        let mut guard = CURRENT_EMITTER.write();
        if Some(ctx.address()) == *guard {
            *guard = None;
        }
    }

    fn handle_text(&mut self, ctx: &mut DanmakuEmitterContext, text: &str) {
        if text.trim() == EMITTER_SECRET {
            tracing::debug!(target: "DanmakuEmitter", "emitter connected");
            *CURRENT_EMITTER.write() = Some(ctx.address());
        }
    }

    fn handle_binary(&mut self, _: &mut DanmakuEmitterContext, _: Bytes) {}
}

impl Handler<BroadcastDanmaku> for DanmakuEmitterWs {
    type Result = ();

    fn handle(&mut self, msg: BroadcastDanmaku, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}
