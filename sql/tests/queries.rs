#[cfg(test)]
mod tests {
    use sql::storage::BitCask;
    use test_each_file::test_each_path;

    test_each_path! { in "sql/tests/testscripts/queries" as math_expressions => test_goldenscript }

    fn test_goldenscript(path: &std::path::Path) {
        let tempdir = tempfile::TempDir::with_prefix("db").expect("tempdir creation failed");
        let bitcask = BitCask::new(tempdir.path()).expect("bitcask creation failed");
    }

    struct SQLRunner<'a> {
        engine: &'a Local,
    }
}
