#[macro_export]
macro_rules! profile {
    ($s:expr) => {
        #[cfg(feature = "profiling")]
        let span = span!(tracing::Level::INFO, $s);
        #[cfg(feature = "profiling")]
        let _enter = span.enter();
    };
}

#[cfg(feature = "profiling")]
pub(crate) use profile;
