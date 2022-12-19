use std::panic;
use tracing::{error, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt};
use tracing_unwrap::OptionExt;

pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::daily("src/engine/logs", "engine.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let offset_time = fmt::time::OffsetTime::new(
        time::UtcOffset::current_local_offset().unwrap(),
        time::macros::format_description!("[hour]:[minute]:[second]"),
    );

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::default().add_directive(
            #[cfg(not(feature = "shipping"))]
            Level::DEBUG.into(),
            #[cfg(feature = "shipping")]
            Level::ERROR.into(),
        ))
        .with(
            fmt::Layer::new()
                .pretty()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_thread_names(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true)
                .with_timer(offset_time.clone()),
        );

    #[cfg(feature = "validation")]
    let subscriber = subscriber
        .with(
            fmt::Layer::new()
                .pretty()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_thread_names(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true)
                .with_timer(offset_time),
        )
        .with(tracing_tracy::TracyLayer::new());

    #[cfg(feature = "profiling")]
    let subscriber = subscriber.with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber).expect("Failed to init logging.");

    panic::set_hook(Box::new(|args| {
        if let Some(message) = args.message() {
            let location = args.location().unwrap_or_log();
            error!(
                "{message}\n  Location: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
    }));

    guard
}
