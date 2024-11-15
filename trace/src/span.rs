use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::{
    GlobalCollect, SpanContext,
    collector::SpanId,
    local::{
        local_collector::LOCAL_SPAN_STACK,
        local_span_stack::{LocalSpanStack, SpanLineHandle},
        raw_span::RawSpan,
    },
};

pub struct Span {
    inner: Option<SpanInner>,
}

impl Span {
    pub fn noop() -> Self {
        Self { inner: None }
    }

    pub fn root(name: impl Into<Cow<'static, str>>, context: SpanContext) -> Self {
        let collect = GlobalCollect {};
        let raw_span = RawSpan::start_with(context.span_id, SpanId::default(), name.into());
        let inner = SpanInner { collect, raw_span };
        Self { inner: Some(inner) }
    }

    pub fn set_local_parent(&self) -> LocalParentGuard {
        LOCAL_SPAN_STACK
            .try_with(|s| self.attach_into_stack(s))
            .unwrap_or_default()
    }

    fn attach_into_stack(&self, stack: &Rc<RefCell<LocalSpanStack>>) -> LocalParentGuard {
        self.inner
            .as_ref()
            .map(move |inner| inner.capture_local_spans(stack.clone()))
            .unwrap_or_else(LocalParentGuard::noop)
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some(mut inner) = self.inner.take() {
            let collect = inner.collect.clone();
            inner.raw_span.end();
            collect.send_command(vec![inner.raw_span]);
        }
    }
}

pub struct SpanInner {
    collect: GlobalCollect,
    raw_span: RawSpan,
}

impl SpanInner {
    fn capture_local_spans(&self, stack: Rc<RefCell<LocalSpanStack>>) -> LocalParentGuard {
        let span_line_handler = {
            let stack = &mut (*stack).borrow_mut();
            stack.register_span_line(Some(self.raw_span.span_id))
        };

        let inner = LocalParentGuardInner {
            stack,
            span_line_handler: span_line_handler.unwrap_or_default(),
            collect: self.collect.clone(),
        };
        LocalParentGuard { inner: Some(inner) }
    }
}

#[derive(Default)]
pub struct LocalParentGuard {
    inner: Option<LocalParentGuardInner>,
}

struct LocalParentGuardInner {
    stack: Rc<RefCell<LocalSpanStack>>,
    span_line_handler: SpanLineHandle,
    collect: GlobalCollect,
}

impl LocalParentGuard {
    pub fn noop() -> Self {
        Self { inner: None }
    }

    pub fn new(
        collect: GlobalCollect,
        span_line_handle: SpanLineHandle,
        stack: Rc<RefCell<LocalSpanStack>>,
    ) -> Self {
        let inner = Some(LocalParentGuardInner {
            stack,
            span_line_handler: span_line_handle,
            collect,
        });
        Self { inner }
    }
}

impl Drop for LocalParentGuard {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            let stack = &mut (*inner.stack).borrow_mut();
            if let Some(spans) = stack.unregister_and_collect(inner.span_line_handler) {
                inner.collect.send_command(spans);
            }
        }
    }
}
