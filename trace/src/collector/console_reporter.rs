use super::{Reporter, SpanRecord};

#[derive(Default)]
pub struct ConsoleReporter {}

impl Reporter for ConsoleReporter {
    fn report(&mut self, spans: Vec<SpanRecord>) {
        for span in spans {
            eprintln!("{:?}", span);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_reporter() {
        let mut reporter = ConsoleReporter::default();
        reporter.report(vec![SpanRecord {
            trace_id: 1,
            span_id: 2,
            parent_id: 0,
            begin_time_unix_ns: 0,
            duration_ns: 0,
            name: "test".into(),
            properties: vec![("key".into(), "value".into())],
        }]);
    }
}
