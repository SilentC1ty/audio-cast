use crate::protocol::packet::AudioPacket;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const SAMPLE_RATE: u32 = 48000;
const FRAME_DURATION_MS: u32 = 10;
const SAMPLES_PER_FRAME: u32 = SAMPLE_RATE * FRAME_DURATION_MS / 1000; // 480
const INNER_PACKET_EMPTY: f32 = -1.0;

#[derive(Debug, Clone)]
pub struct JitterPacket {
    pub sequence: u32,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}

impl PartialEq for JitterPacket {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl Eq for JitterPacket {}

impl PartialOrd for JitterPacket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JitterPacket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl From<AudioPacket> for JitterPacket {
    fn from(p: AudioPacket) -> Self {
        JitterPacket {
            sequence: p.sequence,
            timestamp: p.timestamp,
            payload: p.payload,
        }
    }
}

/// 抖动缓冲区，用于平滑 Wi-Fi 网络抖动
///
/// 使用最小堆按 timestamp 排序，支持 60-120ms 动态调整。
pub struct JitterBuffer {
    heap: BinaryHeap<Reverse<JitterPacket>>,
    target_ms: u32,
    accumulated_ms: u32,
    last_seq: u32,
    packets_received: u64,
    packets_lost: u64,
    consec_lost: u32,
}

impl JitterBuffer {
    pub fn new(target_ms: u32) -> Self {
        JitterBuffer {
            heap: BinaryHeap::new(),
            target_ms: target_ms.clamp(60, 120),
            accumulated_ms: 0,
            last_seq: 0,
            packets_received: 0,
            packets_lost: 0,
            consec_lost: 0,
        }
    }

    /// 插入一个数据包
    pub fn push(&mut self, packet: AudioPacket) {
        self.packets_received += 1;

        // 丢包检测
        if self.packets_received > 1 {
            let gap = packet.sequence.wrapping_sub(self.last_seq);
            if gap > 1 {
                let lost = (gap - 1) as u64;
                self.packets_lost += lost;
                self.consec_lost += lost as u32;
            } else {
                self.consec_lost = 0;
            }
        }
        self.last_seq = packet.sequence;

        self.heap.push(Reverse(JitterPacket::from(packet)));
    }

    /// 弹出一个已就绪的数据包
    pub fn pop(&mut self) -> Option<JitterPacket> {
        if !self.is_ready() {
            return None;
        }

        // 移除 heap 中的空洞：跳过 timestamp 为 INNER_PACKET_EMPTY 的占位
        while let Some(peek) = self.heap.peek() {
            if peek.0.timestamp == INNER_PACKET_EMPTY as u64 {
                self.heap.pop();
            } else {
                break;
            }
        }

        self.heap.pop().map(|r| {
            self.accumulated_ms = self
                .accumulated_ms
                .saturating_sub(FRAME_DURATION_MS);
            r.0
        })
    }

    /// 缓冲区是否已积累足够数据
    pub fn is_ready(&self) -> bool {
        self.accumulated_ms >= self.target_ms
    }

    /// 累积缓冲时长
    pub fn accumulate(&mut self, frame_ms: u32) {
        self.accumulated_ms = self
            .accumulated_ms
            .saturating_add(frame_ms)
            .min(200);
    }

    /// 更新目标缓冲时长（ms）
    pub fn set_target_ms(&mut self, ms: u32) {
        self.target_ms = ms.clamp(60, 120);
    }

    /// 获取当前缓冲数据量对应的时长
    pub fn current_ms(&self) -> u32 {
        self.accumulated_ms
    }

    pub fn target_ms(&self) -> u32 {
        self.target_ms
    }

    /// 丢包统计
    pub fn packet_loss_rate(&self) -> f32 {
        if self.packets_received == 0 {
            return 0.0;
        }
        self.packets_lost as f32 / (self.packets_received + self.packets_lost) as f32
    }

    pub fn packets_received(&self) -> u64 {
        self.packets_received
    }

    pub fn packets_lost(&self) -> u64 {
        self.packets_lost
    }

    /// 自上次弹包后是否有缓冲区欠载（underflow）
    pub fn has_underflow(&self) -> bool {
        self.heap.is_empty() && self.accumulated_ms < self.target_ms
    }

    /// 缓冲区大小
    pub fn packet_count(&self) -> usize {
        self.heap.len()
    }

    /// 连续丢包数
    pub fn consecutive_losses(&self) -> u32 {
        self.consec_lost
    }

    /// 重置连续丢包
    pub fn reset_consecutive_losses(&mut self) {
        self.consec_lost = 0;
    }
}
