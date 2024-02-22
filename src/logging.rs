use eyre::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, prelude::__tracing_subscriber_Layer, util::SubscriberInitExt,
    EnvFilter, Registry,
};
use windows::Win32::Foundation::HINSTANCE;

use crate::paths::get_dll_dir;

/// Setup logging for the plugin
pub fn setup_logging(module: HINSTANCE) -> Result<()> {
    // get the file path to `<path_to_my_dll_folder>\`
    let dll_dir = get_dll_dir(module)?;

    let var = "NMS_LOG";

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var(var)
        .from_env_lossy();

    if cfg!(debug_assertions) {
        fmt::Subscriber::builder()
            .with_ansi(true)
            .without_time()
            .with_env_filter(env_filter)
            .init();
    } else {
        let file_appender = tracing_appender::rolling::never(dll_dir, "native-memory-scripter.log");

        let log_layer = tracing_subscriber::fmt::Layer::default()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_filter(env_filter);

        let env_filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .with_env_var(var)
            .from_env_lossy();

        let stdout_layer = tracing_subscriber::fmt::Layer::default()
            .without_time()
            .with_ansi(true)
            .with_filter(env_filter);

        Registry::default()
            .with(log_layer)
            .with(stdout_layer)
            .init();
    }

    Ok(())
}
