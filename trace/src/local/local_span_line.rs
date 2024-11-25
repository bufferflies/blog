use std::borrow::Cow;

use super::raw_span::RawSpan;
use crate::collector::SpanId;

pub type SpanHandle = usize;

pub struct SpanLine {
    span_queue: Vec<RawSpan>,
    epoch: usize,
    // the next parent span id must be one of the span id in the span_queue
    // it follows the stack semantics
    next_parent_span_id: Option<SpanId>,
}

impl SpanLine {
    pub fn new(capacity: usize, epoch: usize, parent_span_id: Option<SpanId>) -> Self {
        Self {
            span_queue: Vec::with_capacity(capacity),
            epoch,
            next_parent_span_id: parent_span_id,
        }
    }

    pub fn start_span(&mut self, name: impl Into<Cow<'static, str>>) -> Option<SpanHandle> {
        if self.span_queue.len() >= self.span_queue.capacity() {
            return None;
        }
        let span_id = crate::collector::next_span_id();
        let span = RawSpan::start_with(
            span_id,
            self.next_parent_span_id.unwrap_or_default(),
            name.into(),
        );
        self.next_parent_span_id = Some(span_id);
        let handle = self.span_queue.len();
        self.span_queue.push(span);
        Some(handle)
    }

    pub fn end_span(&mut self, handle: SpanHandle) {
        debug_assert!(handle < self.span_queue.len());
        debug_assert_eq!(
            self.next_parent_span_id,
            Some(self.span_queue[handle].span_id)
        );

        let span = self.span_queue.get_mut(handle).unwrap();
        span.end();
        self.next_parent_span_id = Some(span.parent_id).filter(|id| *id != SpanId::default());
    }

    pub fn current_epoch(&self) -> usize {
        self.epoch
    }

    #[inline]
    pub fn collect(self) -> Option<Vec<RawSpan>> {
        Some(self.span_queue)
    }

    pub fn first(&self) -> Option<&RawSpan> {
        self.span_queue.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_span_id() {
        let parent_span_id = 10;
        let mut span_line = SpanLine::new(10, 0, Some(parent_span_id));
        let handle = span_line.start_span("test");
        assert_eq!(handle, Some(0));
        span_line.end_span(0);
        let first_span = span_line.first().unwrap();
        assert_eq!(first_span.parent_id, parent_span_id);
        assert_ne!(first_span.span_id, 0)
    }

    #[test]
    fn serial_span_id() {
        let cap = 10;
        let mut span_line = SpanLine::new(cap, 0, None);
        for i in 0..cap {
            span_line.start_span(format!("test_{}", i));
        }
        let trace = span_line.collect().unwrap();
        for i in 1..cap {
            assert_eq!(trace[i].parent_id, trace[i - 1].span_id);
        }
    }

    #[test]
    fn parallel_span_id() {
        let cap = 10;
        let mut span_line = SpanLine::new(cap, 0, None);
        for i in 0..cap {
            let handle = span_line.start_span(format!("test_{}", i));
            span_line.end_span(handle.unwrap());
        }
        let trace = span_line.collect().unwrap();
        for i in 0..cap {
            assert_eq!(trace[i].parent_id, 0);
        }
    }
}
