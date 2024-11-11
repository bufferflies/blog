pub mod collector;
pub mod future;
pub mod local;
pub mod span;

pub use collector::{
    SpanContext, config::Config, console_reporter::ConsoleReporter, flush,
    global_collector::GlobalCollect, id, set_reporter,
};
pub use local::local_span::LocalSpan;
pub use span::Span;
