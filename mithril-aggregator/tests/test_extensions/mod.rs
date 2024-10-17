// Avoid clippy warnings generated by tests that doesn't use every tests_extensions (since each test
// is a different compilation target).
#![allow(dead_code)]

#[macro_use]
pub mod runtime_tester;
#[macro_use]
pub mod utilities;
pub mod aggregator_observer;
mod expected_certificate;
mod metrics_tester;

pub use aggregator_observer::AggregatorObserver;
pub use expected_certificate::ExpectedCertificate;
// There are several tests where it's not necessary to verify the metrics
#[allow(unused_imports)]
pub use metrics_tester::ExpectedMetrics;
pub use metrics_tester::MetricsVerifier;
pub use runtime_tester::RuntimeTester;
