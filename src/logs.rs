use spdlog::{
    formatter::{self, FullFormatter, PatternFormatter, pattern},
    prelude::*,
    sink::{AsyncPoolSink, FileSink, RotatingFileSink, Sink, StdStream, StdStreamSink},
};
use std::sync::Arc;
pub fn setup_logger() {
    // maybe later add rotating logs support
    let path = "logs/log.log";

    let formatter = Box::new(PatternFormatter::new(pattern!(
        "{time} [{^{level}}] {payload}{eol}"
    )));

    let file_sink = Arc::new(
        RotatingFileSink::builder()
            .base_path(path)
            .rotation_policy(spdlog::sink::RotationPolicy::Daily { hour: 0, minute: 0 })
            .formatter(formatter.clone())
            .build()
            .unwrap(),
    ) as Arc<dyn Sink>;

    let std_stream_sink = Arc::new(
        StdStreamSink::builder()
            .std_stream(StdStream::Stdout)
            .formatter(formatter.clone())
            .build()
            .unwrap(),
    ) as Arc<dyn Sink>;

    // AsyncPoolSink is a combined sink which wraps other sinks
    let async_pool_sink = Arc::new(
        AsyncPoolSink::builder()
            .sinks([file_sink, std_stream_sink].into_iter())
            .build()
            .unwrap(),
    );

    let async_logger = Arc::new(
        Logger::builder()
            .sink(async_pool_sink)
            .flush_level_filter(LevelFilter::All)
            .build()
            .unwrap(),
    );
    spdlog::set_default_logger(async_logger);

    info!("Init log!");
}
