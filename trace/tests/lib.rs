use serial_test::serial;

#[test]
#[serial]
fn test_logging() {
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
                    "[{}] {} {}",
                    record.level(),
                    current.trace_id.0,
                    record.args()
                )
            } else {
                writeln!(buf, "[{}] {}", record.level(), record.args())
            }
        })
        .filter_level(log::LevelFilter::Debug)
        .init();

    let span = SpanContext::random();

    let f = async {
        let jhs = { parallel_job() };

        other_job().await;

        for jh in jhs {
            jh.await.unwrap();
        }
    }
    .in_span(span);

    tokio::spawn(f).await.unwrap();
}
