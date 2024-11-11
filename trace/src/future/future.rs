use std::{cell::RefCell, rc::Rc, task::Poll};

thread_local! {
    pub static LOCAL_SPAN_STACK: Rc<RefCell<LocalSpanStack>> = Rc::new(RefCell::new(LocalSpanStack::new()));
}

struct SpanContext;

impl SpanContext {
    pub fn trace_id() -> Option<TraceId> {
        let stack = LOCAL_SPAN_STACK.try_with(Rc::clone).ok()?;
        let trace_id = stack.borrow().trace_id();
        Some(trace_id)
    }

    pub fn root(id: Option<TraceId>) -> LocalSpanStack {
        let trace_id = id.unwrap_or_else(|| TraceId(rand::random()));
        LOCAL_SPAN_STACK
            .try_with(|stack| stack.borrow_mut().set_trace_id(trace_id))
            .unwrap();
        LocalSpanStack { trace_id }
    }
}

#[derive(Debug)]
pub struct LocalSpanStack {
    trace_id: TraceId,
}

impl LocalSpanStack {
    pub fn new() -> Self {
        Self {
            trace_id: TraceId(0),
        }
    }

    pub fn set_trace_id(&mut self, trace_id: TraceId) {
        self.trace_id = trace_id;
    }

    pub fn trace_id(&self) -> TraceId {
        self.trace_id
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct TraceId(pub u128);

#[pin_project::pin_project]
pub struct TrackedFuture<T> {
    #[pin]
    pub inner: T,
    pub trace_id: Option<TraceId>,
}

impl<T> TrackedFuture<T> {
    pub fn new(f: T) -> Self {
        let trace_id = SpanContext::trace_id();
        Self { inner: f, trace_id }
    }
}

impl<T: std::future::Future> std::future::Future for TrackedFuture<T> {
    type Output = T::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let _guard = this.trace_id.as_ref().map(|trace_id| {
            LOCAL_SPAN_STACK.try_with(|stack| stack.borrow_mut().set_trace_id(trace_id.clone()))
        });
        #[cfg(test)]
        log::info!("poll once");
        let res = this.inner.poll(cx);

        match res {
            r @ Poll::Pending => r,
            other => {
                this.trace_id.take();
                other
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn parallel_job() -> Vec<tokio::task::JoinHandle<()>> {
        let mut v = Vec::with_capacity(4);
        for i in 0..4 {
            v.push(tokio::spawn(TrackedFuture::new(iter_job(i))));
        }
        v
    }

    async fn iter_job(iter: u64) {
        std::thread::sleep(std::time::Duration::from_millis(iter * 10));
        tokio::task::yield_now().await;
        other_job().await;
    }

    async fn other_job() {
        log::info!("begin other job task");
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        log::info!("sleep one sec");
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        log::info!("sleep one finish");
    }

    #[tokio::test]
    async fn test_logging() {
        env_logger::Builder::from_default_env()
            .format(|buf, record| {
                // Convert every log to an event in the current local parent span
                // Event::add_to_local_parent(record.level().as_str(), || {
                //     [("message".into(), record.args().to_string().into())]
                // });

                // Attach the current trace id to the log message
                if let Some(current) = SpanContext::trace_id() {
                    writeln!(
                        buf,
                        "[{}] [{}] {}",
                        record.level(),
                        current.0,
                        record.args()
                    )
                } else {
                    writeln!(buf, "[{}] [{}]", record.level(), record.args())
                }
            })
            .filter_level(log::LevelFilter::Debug)
            .init();

        let span = SpanContext::root(None);
        log::info!("begin sub task,span:{span:?}");
        let f = async {
            let jhs = { parallel_job() };

            other_job().await;

            for jh in jhs {
                jh.await.unwrap();
            }
        };

        tokio::spawn(TrackedFuture::new(f)).await.unwrap();
        log::info!("finish sub task");
    }
}
