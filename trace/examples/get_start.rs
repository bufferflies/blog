use trace::{Config, ConsoleReporter, LocalSpan, SpanContext, flush, set_reporter, span::Span};

fn main() {
    set_reporter(ConsoleReporter::default(), Config::default());
    {
        let parent = SpanContext::random();
        let root = Span::root("root", parent);
        let _local_parent_guard = root.set_local_parent();
        span();
    }
    flush();
}

fn span() {
    let _local_span = LocalSpan::enter_with_local_parent("child");
    std::thread::sleep(std::time::Duration::from_secs(1));
}
