use local_reader_lib::domain::regex_guard::{run_with_timeout, GuardOutcome};
use std::time::Duration;

#[test]
fn malicious_backtracking_is_cut_off() {
    let outcome = run_with_timeout(Duration::from_millis(100), || {
        std::thread::sleep(Duration::from_millis(2000));
        true
    });
    assert_eq!(outcome, GuardOutcome::Timeout);
}

#[test]
fn normal_match_returns_value() {
    let outcome = run_with_timeout(Duration::from_millis(1000), || 1 + 1);
    assert_eq!(outcome, GuardOutcome::Ok(2));
}
