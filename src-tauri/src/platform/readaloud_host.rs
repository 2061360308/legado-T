//! 朗读宿主事件与队列联动骨架（T2.2 spike）。
//!
//! 说明：正式实现中，段落队列、翻页联动、定时与通知控制在此模块与 UI 之间解耦。

use crate::platform::tts::AudioEvent;

/// 宿主侧朗读事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AloudEvent {
    /// 当前段落朗读完成，推进到下一段落。
    ParagraphDone { index: usize },
    /// 朗读整章完成。
    ChapterDone { chapter_index: usize },
    /// 音频系统事件（焦点/耳机/通知）。
    Audio(AudioEvent),
}

/// 段落队列最小模型。
#[derive(Debug, Default)]
pub struct ParagraphQueue {
    cursor: usize,
    total: usize,
}

impl ParagraphQueue {
    pub fn new(total: usize) -> Self {
        Self { cursor: 0, total }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn advance(&mut self) -> Option<usize> {
        if self.cursor + 1 < self.total {
            self.cursor += 1;
            Some(self.cursor)
        } else {
            None
        }
    }
}
