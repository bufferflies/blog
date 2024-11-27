#[cfg(test)]
mod tests {
    use std::{collections::HashMap, error::Error, fmt::Write};

    use itertools::Itertools;
    use sql::{
        Parser, Planner,
        engine::{Engine, Local, Session, StatementResult},
        storage::{self, BitCask},
    };
    use test_each_file::test_each_path;

    test_each_path! { in "sql/tests/testscripts/queries" as math_expressions => test_goldenscript }

    fn test_goldenscript(path: &std::path::Path) {
        let tempdir = tempfile::TempDir::with_prefix("db").expect("tempdir creation failed");
        let bitcask =
            BitCask::new(tempdir.path().join("bitcask")).expect("bitcask creation failed");
        let engine = Local::new(bitcask);
        let mut runner = SQLRunner::new(&engine);
        goldenscript::run(&mut runner, path).expect("goldenscript failed");
    }

    struct SQLRunner<'a> {
        engine: &'a TestEngine,
        sessions: HashMap<String, Session<'a, TestEngine>>,
    }

    type TestEngine = Local<storage::BitCask>;

    impl<'a> SQLRunner<'a> {
        fn new(engine: &'a TestEngine) -> Self {
            Self {
                engine,
                sessions: HashMap::new(),
            }
        }
    }

    impl<'a> goldenscript::Runner for SQLRunner<'a> {
        fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
            let mut output = String::new();
            let prefix = command.prefix.clone().unwrap_or_default();
            let session = self
                .sessions
                .entry(prefix)
                .or_insert_with(|| self.engine.session());
            if !command.args.is_empty() {
                return Err("SQL statements should be given as command with no args".into());
            }
            let input = &command.name;
            let mut tags = command.tags.clone();
            if tags.remove("plan") {
                let ast = Parser::new(input).parse()?;
                let plan =
                    session.with_txn(true, |txn| Planner::new(txn).build(ast)?.optimize())?;
                writeln!(output, "{plan}")?;
            }
            // Execute the statement.
            let result = session.execute(input)?;
            match result {
                StatementResult::Select { columns, rows } => {
                    if tags.remove("header") {
                        writeln!(output, "{}", columns.into_iter().join(", "))?;
                    }
                    for row in rows {
                        writeln!(output, "row: {row:?}")?;
                    }
                }
                result if tags.remove("result") => writeln!(output, "{result:?}")?,
                _ => {}
            }
            Ok(output)
        }
    }
}
