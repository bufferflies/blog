use std::{borrow::Cow, time::Instant};

use crate::collector::SpanId;

#[derive(Debug, Clone)]
pub struct RawSpan {
    pub span_id: SpanId,
    pub parent_id: SpanId,
    pub begin_instant: Instant,
    pub name: Cow<'static, str>,
    pub properties: Vec<(Cow<'static, str>, Cow<'static, str>)>,

    // Will write this field at post processing
    pub end_instant: Option<Instant>,
}

impl RawSpan {
    pub fn start_with(span_id: SpanId, parent_id: SpanId, name: Cow<'static, str>) -> Self {
        Self {
            span_id,
            parent_id,
            begin_instant: Instant::now(),
            name,
            properties: Vec::new(),
            end_instant: None,
        }
    }

    pub fn end(&mut self) {
        self.end_instant.replace(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_raw_span() {
        let mut span = super::RawSpan::start_with(1, 0, "test".into());
        assert_eq!(span.span_id, 1);
        assert_eq!(span.parent_id, 0);
        assert_eq!(span.name, "test");
        assert_eq!(span.properties.len(), 0);
        assert!(span.end_instant.is_none());
        span.end();
        assert!(span.end_instant.is_some());
    }
}
