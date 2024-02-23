use color_eyre::config::PanicHook;
use eyre::{eyre, Result};
use strip_ansi_escapes::Writer;
use tracing::error;
use tracing_appender::rolling::RollingFileAppender;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    fmt::MakeWriter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};
use windows::Win32::Foundation::HINSTANCE;

use crate::paths::get_dll_dir;

struct StripAnsiWriter((String, String));

impl StripAnsiWriter {
    fn new(dir: &str, filename: &str) -> Self {
        Self((dir.to_owned(), filename.to_owned()))
    }
}

impl<'a> MakeWriter<'a> for StripAnsiWriter {
    type Writer = Writer<RollingFileAppender>;

    fn make_writer(&'a self) -> Self::Writer {
        Writer::new(tracing_appender::rolling::never(&self.0 .0, &self.0 .1))
    }
}

/// Setup logging for the plugin
pub fn setup_logging(module: HINSTANCE) -> Result<()> {
    // get the file path to `<path_to_my_dll_folder>\`
    let dll_dir = get_dll_dir(module).map_err(|e| eyre!("{e}"))?;

    let var = "NMS_LOG";

    let env_filter = EnvFilter::try_from_env(var)
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    if cfg!(debug_assertions) {
        let stdout_layer = tracing_subscriber::fmt::Layer::default()
            .without_time()
            .with_ansi(true)
            .with_target(false)
            .with_filter(env_filter);

        Registry::default()
            .with(stdout_layer)
            .with(ErrorLayer::default())
            .init();
    } else {
        // a log writer which also strips ansi, because panic hook unfortunately outputs ansi into the normal stream
        let log_writer =
            StripAnsiWriter::new(&dll_dir.to_string_lossy(), "native-memory-scripter.log");

        let log_layer = tracing_subscriber::fmt::Layer::default()
            .with_writer(log_writer)
            .with_ansi(false)
            .with_target(false)
            .with_filter(env_filter);

        let env_filter = EnvFilter::try_from_env(var)
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();

        let stdout_layer = tracing_subscriber::fmt::Layer::default()
            .without_time()
            .with_ansi(true)
            .with_target(false)
            .with_filter(env_filter);

        Registry::default()
            .with(stdout_layer)
            .with(log_layer)
            .with(ErrorLayer::default())
            .init();
    }

    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section("consider reporting the bug on github @ https://github.com/MolotovCherry/Native-Memory-Scripter").into_hooks();

    eyre_hook.install()?;

    set_panic_hook(panic_hook);

    Ok(())
}

fn set_panic_hook(hook: PanicHook) {
    // this panic hook makes sure that eyre panic hook gets sent to all tracing layers
    std::panic::set_hook(Box::new(move |info| {
        let panic = hook.panic_report(info).to_string();
        error!("{panic}");
    }))
}
