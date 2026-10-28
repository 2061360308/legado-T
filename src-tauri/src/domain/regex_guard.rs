//! 正则执行超时与熔断（T2.4 spike，ADR-05）。
//!
//! 正式实现使用 Rust `regex`（线性时间）；本模块提供与匹配器无关的超时包装，
//! 便于在任何匹配实现外层施加熔断，并保证进程不崩。

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// 默认熔断超时：3000ms。
pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(3000);

/// 执行结果。
#[derive(Debug, PartialEq, Eq)]
pub enum GuardOutcome<R> {
    Ok(R),
    /// 超过超时上限被判定为违规。
    Timeout,
    Panicked,
}

/// 在独立线程执行 `f`，超过 `timeout` 即返回 `Timeout`，进程不崩。
pub fn run_with_timeout<F, R>(timeout: Duration, f: F) -> GuardOutcome<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(timeout) {
        Ok(v) => {
            let _ = handle.join();
            GuardOutcome::Ok(v)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => GuardOutcome::Timeout,
        Err(mpsc::RecvTimeoutError::Disconnected) => GuardOutcome::Panicked,
    }
}

/// 使用默认超时执行。
pub fn run_default<F, R>(f: F) -> GuardOutcome<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    run_with_timeout(DEFAULT_TIMEOUT, f)
}
