//! 系统 TTS 桥接（T2.2 spike）。
//!
//! 定义与平台无关的朗读后端契约与状态机；系统 TTS 的具体驱动在各平台实现中接入。

/// 朗读状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeakState {
    Idle,
    Speaking,
    Paused,
}

/// 外部事件（音频焦点、耳机、通知栏）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEvent {
    FocusGained,
    FocusLost,
    HeadphonesUnplugged,
    HeadphonesPlugged,
    NotifyPause,
    NotifyResume,
}

/// 朗读后端契约（系统 TTS / 在线合成均实现之）。
pub trait TtsBackend {
    fn speak(&mut self, text: &str) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn pause(&mut self) -> Result<(), String>;
    fn resume(&mut self) -> Result<(), String>;
}

/// 朗读控制器状态机（与 UI 解耦）。
#[derive(Debug, Default)]
pub struct TtsController {
    state: SpeakStateSlot,
}

#[derive(Debug, Default)]
struct SpeakStateSlot(SpeakState);

impl Default for SpeakState {
    fn default() -> Self {
        SpeakState::Idle
    }
}

impl TtsController {
    pub fn state(&self) -> SpeakState {
        self.state.0
    }

    pub fn on_audio_event(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::FocusLost | AudioEvent::HeadphonesUnplugged | AudioEvent::NotifyPause => {
                if self.state.0 == SpeakState::Speaking {
                    self.state.0 = SpeakState::Paused;
                }
            }
            AudioEvent::FocusGained | AudioEvent::HeadphonesPlugged | AudioEvent::NotifyResume => {
                if self.state.0 == SpeakState::Paused {
                    self.state.0 = SpeakState::Speaking;
                }
            }
        }
    }

    pub fn start(&mut self) {
        self.state.0 = SpeakState::Speaking;
    }

    pub fn stop(&mut self) {
        self.state.0 = SpeakState::Idle;
    }
}
