use std::borrow::Cow;

use super::{
    local_span_line::{SpanHandle, SpanLine},
    raw_span::RawSpan,
};
use crate::collector::SpanId;

pub type SpanLineHandle = usize;

pub struct LocalSpanStack {
    span_lines: Vec<SpanLine>,
    next_span_line_epoch: usize,
}

impl LocalSpanStack {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            span_lines: Vec::with_capacity(capacity),
            next_span_line_epoch: 0,
        }
    }

    pub fn enter_span(&mut self, name: impl Into<Cow<'static, str>>) -> Option<SpanHandle> {
        let span_line = self.current_span_line()?;
        span_line.start_span(name)
    }

    pub fn exit_span(&mut self, handle: SpanHandle) {
        if let Some(span_line) = self.current_span_line() {
            span_line.end_span(handle);
        }
    }

    #[inline]
    pub fn current_span_line(&mut self) -> Option<&mut SpanLine> {
        self.span_lines.last_mut()
    }

    pub fn register_span_line(&mut self, parent_span_id: Option<SpanId>) -> Option<SpanHandle> {
        if self.span_lines.len() >= self.span_lines.capacity() {
            return None;
        }

        let epoch = self.next_span_line_epoch;
        self.next_span_line_epoch = self.next_span_line_epoch.wrapping_add(1);
        let span_line = SpanLine::new(self.span_lines.capacity(), epoch, parent_span_id);
        self.span_lines.push(span_line);
        Some(epoch)
    }

    pub fn unregister_and_collect(
        &mut self,
        span_line_handle: SpanLineHandle,
    ) -> Option<Vec<RawSpan>> {
        debug_assert_eq!(
            self.current_span_line().unwrap().current_epoch(),
            span_line_handle,
        );
        let span_line = self.span_lines.pop()?;
        span_line.collect()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_local_span_stack() {
        let mut stack = super::LocalSpanStack::with_capacity(10);
        assert!(stack.enter_span("test").is_none());
        let span_line_handler = stack.register_span_line(None).unwrap();
        let spans = stack.unregister_and_collect(span_line_handler).unwrap();
        assert!(spans.is_empty());

        let span_line_handler = stack.register_span_line(None).unwrap();
        stack.enter_span("test");
        let spans = stack.unregister_and_collect(span_line_handler).unwrap();
        assert_eq!(spans.len(), 1);
    }
}
