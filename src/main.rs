#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::absolute_paths)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::as_conversions)]
#![allow(clippy::cast_lossless)] // u32 -> u64
#![allow(clippy::cast_possible_truncation)] // u64 -> u32
#![allow(clippy::cast_possible_wrap)] // u32 -> i32
#![allow(clippy::cast_sign_loss)] // i32 -> u32
#![allow(clippy::option_if_let_else)]
#![allow(clippy::future_not_send)]
#![allow(clippy::host_endian_bytes)]
#![allow(clippy::implicit_return)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::inline_always)]
#![allow(clippy::integer_division)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::missing_assert_message)]
#![allow(clippy::missing_trait_methods)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_unsafe_ops_per_block)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::single_char_lifetime_names)]
#![allow(clippy::pattern_type_mismatch)]
#![allow(clippy::pub_use)]
#![allow(clippy::question_mark_used)]
#![allow(clippy::ref_patterns)]
#![allow(clippy::self_named_module_files)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::similar_names)]
#![allow(clippy::single_call_fn)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::string_add)]
#![allow(clippy::unseparated_literal_suffix)]
#![allow(clippy::wildcard_enum_match_arm)]
#![allow(internal_features)]
#![allow(non_snake_case)]
#![allow(soft_unstable)]
#![feature(associated_type_defaults)]
#![feature(const_collections_with_hasher)]
#![feature(const_format_args)]
#![feature(core_intrinsics)]
#![feature(cow_is_borrowed)]
#![feature(error_type_id)]
#![feature(future_join)]
#![feature(iter_partition_in_place)]
#![feature(lazy_cell)]
#![feature(let_chains)]
#![feature(map_try_insert)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_uninit_array)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(ptr_sub_ptr)]
#![feature(slice_split_at_unchecked)]
#![feature(stmt_expr_attributes)]
#![feature(try_blocks)]
#![feature(try_trait_v2)]
#![feature(utf8_chunks)]

mod api;
mod libs;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{middleware, App, HttpServer};

    libs::logger::init();

    libs::db::init_db().await;

    tokio::task::spawn(libs::db::expired_token_cleaner());

    let server = HttpServer::new(move || {
        App::new()
        .wrap(middleware::NormalizePath::new(
            middleware::TrailingSlash::MergeOnly,
        ))
        .wrap(middleware::Logger::new(
            r#"%{009f34034b761c32384fde345378c488efc18c59}i %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#
        ))
        .service(api::chagpt::chagpt)
        .service(api::chagpt::chagpt_admin)
        .service(api::chagpt::emitter)
    });

    server.bind_uds("backend.sock")?.run().await
}
