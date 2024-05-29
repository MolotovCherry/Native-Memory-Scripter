use std::panic;

use color_eyre::config::PanicHook;
use eyre::Result;
use strip_ansi_escapes::Writer;
use tracing::{error, level_filters::LevelFilter};
use tracing_appender::rolling::RollingFileAppender;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::Targets, fmt::MakeWriter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
    Registry,
};
use windows::Win32::Foundation::HINSTANCE;

use crate::{config::Config, paths::get_dll_dir};

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
pub fn setup_logging(module: HINSTANCE, config: &Config) -> Result<()> {
    // get the file path to `<path_to_my_dll_folder>\`
    let dll_dir = get_dll_dir(module)?;

    let var = "NMS_LOG";

    // ignore this cause it always spits out errors we don't want
    let targets = Targets::new()
        .with_target("rustpython_vm::frame", LevelFilter::ERROR)
        .with_default(LevelFilter::TRACE);

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var(var)
        .with_regex(false)
        .parse(&config.log.level)?;

    if cfg!(debug_assertions) {
        let stdout_layer = tracing_subscriber::fmt::Layer::default()
            .without_time()
            .with_ansi(true)
            .with_target(config.log.targets);

        Registry::default()
            .with(stdout_layer)
            .with(ErrorLayer::default())
            .with(targets)
            .with(env_filter)
            .init();
    } else {
        // a log writer which also strips ansi, because panic hook unfortunately outputs ansi into the normal stream
        let log_writer =
            StripAnsiWriter::new(&dll_dir.to_string_lossy(), "native-memory-scripter.log");

        let log_layer = tracing_subscriber::fmt::Layer::default()
            .with_writer(log_writer)
            .with_ansi(false)
            .with_target(config.log.targets);

        let stdout_layer = tracing_subscriber::fmt::Layer::default()
            .without_time()
            .with_ansi(true)
            .with_target(config.log.targets);

        Registry::default()
            .with(stdout_layer)
            .with(log_layer)
            .with(ErrorLayer::default())
            .with(targets)
            .with(env_filter)
            .init();
    }

    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
        .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
        .into_hooks();

    eyre_hook.install()?;
    set_panic_hook(panic_hook);

    Ok(())
}

fn set_panic_hook(hook: PanicHook) {
    // this panic hook makes sure that eyre panic hook gets sent to all tracing layers
    panic::set_hook(Box::new(move |info| {
        let panic = hook.panic_report(info);
        error!("{panic}");
    }))
}
