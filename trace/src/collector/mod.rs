pub mod config;
pub mod console_reporter;
pub mod global_collector;
pub mod id;

use std::{borrow::Cow, rc::Rc};
pub type TraceId = u64;
pub type SpanId = u64;
pub use global_collector::{flush, set_reporter};
pub use id::next_span_id;

use crate::local::local_collector::LOCAL_SPAN_STACK;

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

#[derive(Clone, Copy, Debug, Default)]
pub struct SpanContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
}

impl SpanContext {
    pub fn new(trace_id: TraceId, span_id: SpanId) -> Self {
        Self { trace_id, span_id }
    }

    pub fn random() -> Self {
        Self {
            trace_id: rand::random(),
            span_id: rand::random(),
        }
    }

    pub fn current_local_parent() -> Option<Self> {
        let stack = LOCAL_SPAN_STACK.try_with(Rc::clone).ok()?;

        let mut stack = stack.borrow_mut();
        let span_line = stack.current_span_line()?;
        let first_raw_span = span_line.first()?;

        Some(Self {
            trace_id: 0,
            span_id: first_raw_span.span_id,
        })
    }
}
