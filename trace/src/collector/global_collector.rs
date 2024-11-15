use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Mutex, mpsc},
    time::Instant,
};

use super::{Reporter, SpanRecord, config::Config, id::next_collect_id};
use crate::local::raw_span::RawSpan;

static GLOBAL_COLLECTOR: Mutex<Option<GlobalCollector>> = Mutex::new(None);
type LocalCollector = Rc<RefCell<Option<mpsc::Sender<CollectCommand>>>>;

thread_local! {
    static COMMAND_SENDER: LocalCollector = {
        let tx=if let Some(collector)=GLOBAL_COLLECTOR.lock().unwrap().as_mut(){
            Some(collector.tx.clone())
         }else{
             None
         };
        Rc::new(RefCell::new(tx))
    };
}

fn send_command(cmd: CollectCommand) {
    COMMAND_SENDER.with(|sender| {
        if let Some(tx) = sender.borrow().as_ref() {
            tx.send(cmd).unwrap();
        }
    });
}

struct CollectCommand {
    spans: Vec<SpanRecord>,
}

pub struct GlobalCollector {
    reporter: Option<Box<dyn Reporter>>,
    config: Config,
    rx: mpsc::Receiver<CollectCommand>,
    tx: mpsc::Sender<CollectCommand>,
}

pub fn set_reporter(reporter: impl Reporter, config: Config) {
    GlobalCollector::start(Box::new(reporter), config);
}

impl GlobalCollector {
    fn report(&mut self, spans: Vec<SpanRecord>) {
        if let Some(reporter) = &mut self.reporter {
            reporter.report(spans);
        }
    }

    pub fn start(report: Box<dyn Reporter>, config: Config) {
        let (tx, rx) = mpsc::channel::<CollectCommand>();
        let global_collector = GlobalCollector {
            reporter: Some(report),
            config,
            rx,
            tx,
        };
        GLOBAL_COLLECTOR.lock().unwrap().replace(global_collector);

        std::thread::Builder::new()
            .name("fastrace-global-collector".to_string())
            .spawn(move || {
                loop {
                    let begin_instant = Instant::now();
                    if let Some(collector) = GLOBAL_COLLECTOR.lock().unwrap().as_mut() {
                        collector.handle_commands();
                        std::thread::sleep(
                            collector
                                .config
                                .report_interval
                                .saturating_sub(begin_instant.elapsed()),
                        );
                    }
                }
            })
            .unwrap();
    }

    fn handle_commands(&mut self) {
        let mut spans = Vec::new();
        while let Ok(command) = self.rx.try_recv() {
            spans.extend(command.spans);
        }
        self.report(spans);
    }
}

#[derive(Default, Clone)]
pub struct GlobalCollect;

impl GlobalCollect {
    pub fn start_collect(&self) -> usize {
        let collect_id = next_collect_id();
        collect_id
    }

    pub fn send_command(&self, spans: Vec<RawSpan>) {
        let span_records = spans
            .into_iter()
            .map(|span| {
                let duration_ns = span
                    .end_instant
                    .map(|end| end.saturating_duration_since(span.begin_instant).as_nanos() as u64);
                SpanRecord {
                    trace_id: 0,
                    span_id: span.span_id,
                    parent_id: span.parent_id,
                    begin_time_unix_ns: span.begin_instant.elapsed().as_nanos() as u64,
                    duration_ns: duration_ns.unwrap_or_default(),
                    name: span.name,
                    properties: span.properties,
                }
            })
            .collect();
        let cmd = CollectCommand {
            spans: span_records,
        };
        send_command(cmd);
    }
}

/// Flushes all pending span records to the reporter immediately.
pub fn flush() {
    // Spawns a new thread to ensure the reporter operates outside the tokio runtime
    // to prevent panic.
    std::thread::Builder::new()
        .name("fastrace-flush".to_string())
        .spawn(move || {
            if let Some(global_collector) = GLOBAL_COLLECTOR.lock().unwrap().as_mut() {
                global_collector.handle_commands();
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::send_command;
    use crate::collector::{config::Config, console_reporter::ConsoleReporter};

    #[test]
    fn test_global_collector() {
        let console_reporter = ConsoleReporter::default();
        super::GlobalCollector::start(Box::new(console_reporter), Config::default());
        let cmd = super::CollectCommand {
            spans: vec![super::SpanRecord {
                trace_id: 1,
                span_id: 2,
                parent_id: 0,
                begin_time_unix_ns: 0,
                duration_ns: 0,
                name: "test".into(),
                properties: vec![("key".into(), "value".into())],
            }],
        };
        send_command(cmd);
        sleep(std::time::Duration::from_secs(1));
    }
}
