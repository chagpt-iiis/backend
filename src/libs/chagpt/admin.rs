use actix::{fut::wrap_future, ActorFutureExt, Addr, AsyncContext, Handler};
use actix_web_actors::ws;
use bytestring::ByteString;
use parking_lot::RwLock;
use serde::Deserialize;

use crate::libs::{
    constants::ADMIN_SECRET,
    ws::{AppWsActor, WsActor},
};

use super::{
    chagpt::ACTORS,
    emitter::CURRENT_EMITTER,
    repertoire::{self, Program, Repertoire},
    Emit,
};

#[derive(Default)]
pub struct ChaGPTAdminActor {
    is_login: bool,
}
pub type ChaGPTAdminWsActor = WsActor<ChaGPTAdminActor>;
pub type ChaGPTAdminContext = ws::WebsocketContext<ChaGPTAdminWsActor>;
type MaybeAddress = Option<Addr<ChaGPTAdminWsActor>>;

pub static CURRENT_ADMIN: RwLock<MaybeAddress> = RwLock::new(None);

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Message {
    #[serde(rename = "repertoire-update")]
    RepUp {
        programs: Vec<Program>,
        current: u32,
    },
    #[serde(rename = "danmaku-checked")]
    DanChk { content: String, color: u32 },
}

impl AppWsActor for ChaGPTAdminActor {
    fn started(&mut self, _ctx: &mut ChaGPTAdminContext, _hash: u64) {
        // TODO
    }

    fn stopped(&mut self, _ctx: &mut ChaGPTAdminContext, _hash: u64) {
        // TODO
    }

    fn handle_text(&mut self, ctx: &mut ChaGPTAdminContext, text: &str) {
        if !self.is_login {
            if text.trim() == ADMIN_SECRET {
                tracing::debug!(target: "ChaGPT-admin", "admin connected");
                self.is_login = true;
                ctx.text(format!("4{ADMIN_SECRET}"));
                *CURRENT_ADMIN.write() = Some(ctx.address());
            }
            return;
        }

        let Ok(msg) = serde_json::from_str::<Message>(text) else {
            return;
        };
        match msg {
            Message::RepUp { programs, current } => {
                ctx.wait(
                    wrap_future(repertoire::update(Repertoire { programs, current })).map(
                        |res, _actor, _ctx| {
                            match res {
                                Err(e) => tracing::warn!(target: "ChaGPT-admin", "failed to update repertoire: {e:?}"),
                                Ok(payload) => {
                                    let payload = Emit(ByteString::from(payload));
                                    let guard = ACTORS.read();
                                    for actor in &*guard {
                                        if let Err(e) = actor.do_send(payload.clone()) {
                                            tracing::error!(target: "repertoire-update-to-client", err = ?e);
                                        }
                                    }
                                }
                            }
                        },
                    ),
                );
            }
            Message::DanChk { content, color } => {
                let Ok(content) = serde_json::to_string(&content) else {
                    return;
                };
                let payload = Emit(ByteString::from(format!(
                    r#"4{{"content":{content},"color":{color}}}"#
                )));
                tracing::debug!("emit {:?}", payload.0);
                if let Some(ref addr) = *CURRENT_EMITTER.read() {
                    if let Err(e) = addr.do_send(payload) {
                        tracing::error!(target: "danmaku-to-emitter", err = ?e);
                    }
                }
            }
        }
    }

    fn handle_binary(&mut self, _ctx: &mut ChaGPTAdminContext, _bin: bytes::Bytes) {
        // TODO
    }
}

impl Handler<Emit> for ChaGPTAdminWsActor {
    type Result = ();

    fn handle(&mut self, msg: Emit, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}
