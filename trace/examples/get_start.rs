use trace::{Config, ConsoleReporter, LocalSpan, SpanContext, flush, set_reporter, span::Span};

fn main() {
    set_reporter(ConsoleReporter::default(), Config::default());
    {
        let parent = SpanContext::random();
        let root = Span::root("root", parent);
        let _local_parent_guard = root.set_local_parent();
        let _local_span = LocalSpan::enter_with_local_parent("child");
    }
    flush();
}

#[test]
fn test_logging() {
    assert_eq!(3, 1 + 2);
}
