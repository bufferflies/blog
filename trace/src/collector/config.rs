use std::{default, time::Duration};

use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Default)]
pub struct Config {
    pub max_spans_per_trace: Option<usize>,
    #[derivative(Default(value = "std::time::Duration::from_secs(1)"))]
    pub report_interval: Duration,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let config = Config::default();
        assert_eq!(config.max_spans_per_trace, None);
        assert_eq!(config.report_interval, Duration::from_secs(1));
    }
}
