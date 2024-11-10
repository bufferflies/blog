use std::borrow::Cow;

mod config;
mod console_reporter;
mod global_collector;

pub type TraceId = u64;
pub type SpanId = u64;

#[derive(Debug)]
pub struct SpanRecord {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_id: SpanId,
    pub begin_time_unix_ns: u64,
    pub duration_ns: u64,
    pub name: Cow<'static, str>,
    pub properties: Vec<(Cow<'static, str>, Cow<'static, str>)>,
}

pub trait Reporter: Send + 'static {
    /// Reports a batch of spans to a remote service.
    fn report(&mut self, spans: Vec<SpanRecord>);
}
