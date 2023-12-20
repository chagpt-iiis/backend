use actix_web_actors::ws;

use crate::libs::ws::{AppWsActor, WsActor};

pub struct ChaGPTAdminActor;
pub type ChaGPTAdminWsActor = WsActor<ChaGPTAdminActor>;
pub type ChaGPTAdminContext = ws::WebsocketContext<ChaGPTAdminWsActor>;

impl AppWsActor for ChaGPTAdminActor {
    fn started(&mut self, _ctx: &mut ChaGPTAdminContext, _hash: u64) {
        // TODO
    }

    fn stopped(&mut self, _ctx: &mut ChaGPTAdminContext, _hash: u64) {
        // TODO
    }

    fn handle_text(&mut self, _ctx: &mut ChaGPTAdminContext, _text: &str) {
        // TODO
    }

    fn handle_binary(&mut self, _ctx: &mut ChaGPTAdminContext, _bin: bytes::Bytes) {
        // TODO
    }
}
