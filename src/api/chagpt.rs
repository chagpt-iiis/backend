use actix_http::ws::Codec;
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::libs::{
    chagpt::admin::ChaGPTAdminActor, chagpt::chagpt::ChaGPTActor, chagpt::emitter::DanmakuEmitter,
    ws::WsActor,
};

#[get("/chagpt")]
pub async fn chagpt(req: HttpRequest, stream: web::Payload) -> actix_web::Result<HttpResponse> {
    let mut res = ws::handshake(&req)?;
    Ok(res.streaming(ws::WebsocketContext::with_codec(
        WsActor::new(ChaGPTActor, true),
        stream,
        Codec::new().max_size(0x7fff_ffff),
    )))
}

#[get("/chagpt-admin")]
pub async fn chagpt_admin(
    req: HttpRequest,
    stream: web::Payload,
) -> actix_web::Result<HttpResponse> {
    let mut res = ws::handshake(&req)?;
    Ok(res.streaming(ws::WebsocketContext::with_codec(
        WsActor::new(ChaGPTAdminActor, true),
        stream,
        Codec::new().max_size(0x7fff_ffff),
    )))
}

#[get("/danmaku")]
pub async fn emitter(req: HttpRequest, stream: web::Payload) -> actix_web::Result<HttpResponse> {
    let mut res = ws::handshake(&req)?;
    Ok(res.streaming(ws::WebsocketContext::with_codec(
        WsActor::new(DanmakuEmitter, true),
        stream,
        Codec::new().max_size(0x7fff_ffff),
    )))
}
