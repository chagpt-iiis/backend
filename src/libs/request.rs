use actix_http::Method;
use actix_web::guard::{Guard, GuardContext};

#[allow(non_camel_case_types)]
pub struct POST_or_HEAD;

impl Guard for POST_or_HEAD {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        let method = &ctx.head().method;

        method == Method::POST || method == Method::OPTIONS
    }
}
