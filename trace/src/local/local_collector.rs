use std::{cell::RefCell, rc::Rc};

use super::local_span_stack::{LocalSpanStack, SpanLineHandle};

const DEFAULT_SPAN_STACK_SIZE: usize = 4096;
thread_local! {
    pub static LOCAL_SPAN_STACK: Rc<RefCell<LocalSpanStack>> = Rc::new(RefCell::new(LocalSpanStack::with_capacity(DEFAULT_SPAN_STACK_SIZE)));
}

#[derive(Default)]
pub struct LocalCollector {
    inner: Option<LocalCollectorInner>,
}

struct LocalCollectorInner {
    stack: Rc<RefCell<LocalSpanStack>>,
    span_line_handle: SpanLineHandle,
}

impl LocalCollector {
    fn new(stack: Rc<RefCell<LocalSpanStack>>) -> Self {
        let span_line_handle = {
            let mut stack = stack.borrow_mut();
            stack.register_span_line(None)
        };
        let inner = span_line_handle.map(move |handle| LocalCollectorInner {
            stack,
            span_line_handle: handle,
        });
        Self { inner }
    }
}

impl Drop for LocalCollector {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            let stack = &mut (*inner.stack).borrow_mut();
            stack.unregister_and_collect(inner.span_line_handle);
        }
    }
}
