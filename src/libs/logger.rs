use parking_lot::Once;

static LOGGER_INIT: Once = Once::new();

pub fn init() {
    LOGGER_INIT.call_once(pretty_env_logger::init_timed);
}
