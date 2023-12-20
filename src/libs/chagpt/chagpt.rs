use std::{sync::LazyLock, time::SystemTime};

use actix::{fut::wrap_future, ActorFutureExt, Addr, AsyncContext, Handler};
use actix_web::web::Bytes;
use actix_web_actors::ws;
use ahash::HashSet;
use bytestring::ByteString;
use parking_lot::RwLock;

use super::{
    danmaku::{BroadcastDanmaku, Danmaku, Message},
    emitter::CURRENT_EMITTER,
};
use crate::libs::ws::{AppWsActor, WsActor};

pub struct ChaGPTActor;

pub type ChaGPTWsActor = WsActor<ChaGPTActor>;
pub type ChaGPTContext = ws::WebsocketContext<ChaGPTWsActor>;
type Set = HashSet<Addr<ChaGPTWsActor>>;

static ACTORS: LazyLock<RwLock<Set>> = LazyLock::new(|| RwLock::new(Set::default()));

impl AppWsActor for ChaGPTActor {
    fn started(&mut self, ctx: &mut ChaGPTContext, hash: u64) {
        let mut guard = ACTORS.write();
        if guard.insert(ctx.address()) {
            tracing::debug!(target: "ChaGPT-actor", "\x1b[33mINSERT \x1b[32m{hash:#x}\x1b[33m, size => \x1b[32m{}\x1b[0m", guard.len());
        } else {
            tracing::error!(target: "ChaGPT-actor", "\x1b[1;31mINSERT \x1b[32m{hash:#x}\x1b[31m, size => \x1b[32m{}\x1b[0m", guard.len());
        }
    }

    fn stopped(&mut self, ctx: &mut ChaGPTContext, hash: u64) {
        let mut guard = ACTORS.write();
        if guard.remove(&ctx.address()) {
            tracing::debug!(target: "ChaGPT-actor", "\x1b[33mREMOVE \x1b[32m{hash:#x}\x1b[33m, size => \x1b[32m{}\x1b[0m", guard.len());
        } else {
            tracing::error!(target: "ChaGPT-actor", "\x1b[1;31mREMOVE \x1b[32m{hash:#x}\x1b[31m, size => \x1b[32m{}\x1b[0m", guard.len());
        }
    }

    fn handle_text(&mut self, ctx: &mut ChaGPTContext, text: &str) {
        tracing::debug!(target: "ChaGPT-actor", "handle text with length {}", text.len());

        let Ok(msg): Result<Message, _> = serde_json::from_str(text) else {
            return;
        };
        match msg {
            Message::Propose { content, color } => {
                ctx.wait(wrap_future(Danmaku::insert(content, color)).map(
                    |danmaku: Option<Danmaku>, _actor: &mut ChaGPTWsActor, _ctx| {
                        let Some(danmaku) = danmaku else { return };

                        let timestamp = unsafe {
                            danmaku
                                .time
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_unchecked()
                                .as_millis()
                        };

                        let Ok(content) = serde_json::to_string(&danmaku.content) else {
                            return;
                        };

                        let payload = BroadcastDanmaku(ByteString::from(format!(
                            r#"4{{"type":"danmaku","id":{},"content":{content},"time":{timestamp},"color":{}}}"#,
                            danmaku.id, danmaku.color
                        )));

                        {
                            let guard = ACTORS.read();
                            for actor in &*guard {
                                if let Err(e) = actor.do_send(payload.clone()) {
                                    tracing::error!(target: "ChaGPT-broadcast", err = ?e);
                                }
                            }
                        }
                        if let Some(ref addr) = *CURRENT_EMITTER.read() {
                            if let Err(e) = addr.do_send(payload) {
                                tracing::error!(target: "ChaGPT-broadcast-to-emitter", err = ?e);
                            }
                        }
                    },
                ));
            }
        }
    }

    fn handle_binary(&mut self, _: &mut ChaGPTContext, bin: Bytes) {
        tracing::debug!(target: "ChaGPT-actor", "handle binary with length {}", bin.len());
    }
}

impl Handler<BroadcastDanmaku> for ChaGPTWsActor {
    type Result = ();

    fn handle(&mut self, msg: BroadcastDanmaku, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}
