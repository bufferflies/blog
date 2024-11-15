use std::{borrow::Cow, cell::RefCell, rc::Rc};

use super::{
    local_collector::LOCAL_SPAN_STACK, local_span_line::SpanHandle,
    local_span_stack::LocalSpanStack,
};

#[derive(Default)]
pub struct LocalSpan {
    inner: Option<LocalSpanInner>,
}

impl LocalSpan {
    pub fn enter_with_local_parent(name: impl Into<Cow<'static, str>>) -> Self {
        LOCAL_SPAN_STACK
            .try_with(|stack| Self::enter_with_stack(name, stack.clone()))
            .unwrap_or_default()
    }

    fn enter_with_stack(
        name: impl Into<Cow<'static, str>>,
        stack: Rc<RefCell<LocalSpanStack>>,
    ) -> Self {
        let span_handle = {
            let mut stack = stack.borrow_mut();
            stack.enter_span(name)
        };

        let inner = span_handle.map(|span_handle| LocalSpanInner { stack, span_handle });
        Self { inner }
    }
}

impl Drop for LocalSpan {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            let mut stack = inner.stack.borrow_mut();
            stack.exit_span(inner.span_handle);
        }
    }
}

struct LocalSpanInner {
    stack: Rc<RefCell<LocalSpanStack>>,
    span_handle: SpanHandle,
}

#[cfg(test)]
mod tests {
    use crate::local::local_collector::LOCAL_SPAN_STACK;

    #[test]
    fn test_local_span() {
        let span = super::LocalSpan::enter_with_local_parent("test");
        drop(span);
        LOCAL_SPAN_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            assert!(stack.current_span_line().is_none());
        });
    }
}
