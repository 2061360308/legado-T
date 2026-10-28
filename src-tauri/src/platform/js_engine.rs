//! JS 引擎沙箱（T2.1 spike）。
//!
//! 设计目标（ADR-02）：
//! - 在 Rust 进程内执行社区 JS，注入受控桥接对象，禁文件/网络直连；
//! - 支持执行超时中止；
//! - 引擎实现（rquickjs/boa）在正式实现阶段接入；此处提供可单测的沙箱策略、
//!   静态预检与线程级超时包装。

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// 沙箱策略。
#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    /// 单次执行超时上限。
    pub timeout: Duration,
    /// 是否允许脚本直连网络（默认否，网络只能经受控桥接）。
    pub allow_net: bool,
    /// 是否允许脚本访问文件系统（默认否）。
    pub allow_fs: bool,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(3000),
            allow_net: false,
            allow_fs: false,
        }
    }
}

/// 静态预检中被视为越权的令牌。
const BANNED_TOKENS: [&str; 7] = [
    "require(",
    "process",
    "Deno",
    "XMLHttpRequest",
    "fetch(",
    "readFile",
    "writeFile",
];

#[derive(Debug, Clone)]
pub struct JsEngine {
    policy: SandboxPolicy,
}

impl JsEngine {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self { policy }
    }

    /// 静态预检：拦截明确的越权令牌（不是完备的沙箱，但可快速拒绝明显违规脚本）。
    pub fn precheck(&self, script: &str) -> Result<(), String> {
        for token in BANNED_TOKENS {
            let is_net = token == "fetch(" || token == "XMLHttpRequest";
            let is_fs = token == "readFile" || token == "writeFile";
            if script.contains(token) {
                if is_net && !self.policy.allow_net {
                    return Err(format!("blocked network token: {token}"));
                }
                if is_fs && !self.policy.allow_fs {
                    return Err(format!("blocked filesystem token: {token}"));
                }
                if !is_net && !is_fs {
                    return Err(format!("blocked token: {token}"));
                }
            }
        }
        Ok(())
    }

    /// 在线程中执行任务并施加超时。超时返回 Err（线程无法强杀，需 Worker 兜底）。
    pub fn eval_with_timeout<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(f());
        });
        match rx.recv_timeout(self.policy.timeout) {
            Ok(v) => Ok(v),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err(format!("execution timeout after {:?}", self.policy.timeout))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err("execution channel disconnected".into())
            }
        }
    }
}

impl Default for JsEngine {
    fn default() -> Self {
        Self::new(SandboxPolicy::default())
    }
}
