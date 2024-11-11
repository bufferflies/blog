use std::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering},
};

thread_local! {
    static LOCAL_ID_GENERATOR: Cell<(u32, u32)> = Cell::new((rand::random(), 0))
}

pub type SpanId = u64;

pub fn next_span_id() -> SpanId {
    LOCAL_ID_GENERATOR
        .try_with(|id| {
            let (prefix, mut suffix) = id.get();
            suffix = suffix.wrapping_add(1);
            id.set((prefix, suffix));
            ((prefix as u64) << 32) | (suffix as u64)
        })
        .unwrap_or_else(|_| rand::random())
}

pub type CollectId = usize;

static NEXT_COLLECT_ID: AtomicUsize = AtomicUsize::new(0);

pub fn next_collect_id() -> CollectId {
    NEXT_COLLECT_ID.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_id() {
        let id1 = next_span_id();
        let id2 = next_span_id();
        assert_ne!(id1, id2);
    }
}
