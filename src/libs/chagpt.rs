use bytestring::ByteString;

pub mod admin;
pub mod chagpt;
pub mod danmaku;
pub mod emitter;
pub mod repertoire;

pub async fn init() {
    if let Err(e) = repertoire::init().await {
        tracing::warn!(target: "ChaGPT-init", "failed to init repertoire: {e:?}");
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Emit(pub ByteString);

impl actix::Message for Emit {
    type Result = ();
}
