use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt};

pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::daily("src/engine/logs", "engine.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let offset_time = fmt::time::OffsetTime::new(
        time::UtcOffset::current_local_offset().unwrap(),
        time::macros::format_description!("[hour]:[minute]:[second]"),
    );

    #[cfg(not(feature = "shipping"))]
    let subsciber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::default().add_directive(Level::DEBUG.into()))
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
        )
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
        );

    #[cfg(feature = "shipping")]
    let subsciber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::default().add_directive(Level::ERROR.into()))
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

    tracing::subscriber::set_global_default(subsciber).expect("Failed to init logging.");

    guard
}
