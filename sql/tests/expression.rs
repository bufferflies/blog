#[cfg(test)]
mod tests {
    use std::{error::Error, fmt::Write};

    use sql::{Parser, Planner};
    use test_each_file::test_each_path;

    test_each_path! { in "sql/tests/testscripts/expressions/math" as math_expressions => test_goldenscript_expr }
    test_each_path! { in "sql/tests/testscripts/expressions/logic" as logic_expressions => test_goldenscript_expr }
    test_each_path! { in "sql/tests/testscripts/expressions/compare" as comapre_expressions => test_goldenscript_expr }
    test_each_path! { in "sql/tests/testscripts/expressions/sql" as expressions => test_goldenscript_expr }

    /// Runs expression goldenscripts.
    fn test_goldenscript_expr(path: &std::path::Path) {
        goldenscript::run(&mut ExpressionRunner, path).expect("goldenscript failed")
    }

    struct ExpressionRunner;

    impl goldenscript::Runner for ExpressionRunner {
        fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
            let mut output = String::new();
            if !command.args.is_empty() {
                return Err("Expected no arguments".into());
            }
            let input = &command.name;
            let mut tags = command.tags.clone();
            let mut parser = Parser::new(input);
            let ast = parser.parse_expression()?;
            eprintln!("input: {input} → {ast:?}");
            if let Some(next) = parser.lexer.next().transpose()? {
                return Err(format!("unconsumed token {next}").into());
            }
            let expr = Planner::build_expression(ast)?;
            let value = expr.evaluate(None)?;
            write!(output, "{value}")?;

            // If requested, debug-dump the parsed expression.
            if tags.remove("expr") {
                write!(output, " ← {:?}", expr)?;
            }
            writeln!(output)?;

            // Reject unknown tags.
            if let Some(tag) = tags.iter().next() {
                return Err(format!("unknown tag {tag}").into());
            }

            Ok(output)
        }
    }
}
