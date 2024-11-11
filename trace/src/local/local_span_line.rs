use std::borrow::Cow;

use super::raw_span::RawSpan;
use crate::collector::SpanId;

pub type SpanHandle = usize;

pub struct SpanLine {
    span_queue: Vec<RawSpan>,
    epoch: usize,
    // 记录当前 thread stack 函数调用的父节点
    next_parent_id: Option<SpanId>,
}

impl SpanLine {
    pub fn new(capacity: usize, epoch: usize, parent_span_id: Option<SpanId>) -> Self {
        Self {
            span_queue: Vec::with_capacity(capacity),
            epoch,
            next_parent_id: parent_span_id,
        }
    }

    pub fn start_span(&mut self, name: impl Into<Cow<'static, str>>) -> Option<SpanHandle> {
        if self.span_queue.len() >= self.span_queue.capacity() {
            return None;
        }
        let span_id = crate::collector::next_span_id();
        let span = RawSpan::start_with(
            span_id,
            self.next_parent_id.unwrap_or_default(),
            name.into(),
        );
        self.next_parent_id = Some(span_id);
        let handle = self.span_queue.len();
        self.span_queue.push(span);
        return Some(handle);
    }

    pub fn end_span(&mut self, handle: SpanHandle) {
        debug_assert!(handle < self.span_queue.len());
        debug_assert_eq!(self.next_parent_id, Some(self.span_queue[handle].span_id));

        let span = self.span_queue.get_mut(handle).unwrap();
        span.end();
        self.next_parent_id = Some(span.parent_id).filter(|id| *id != SpanId::default());
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
