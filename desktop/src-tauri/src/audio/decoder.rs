use opus::{Channels, Decoder as OpusDecoder};

pub const SAMPLE_RATE: u32 = 48000;
pub const CHANNELS: u16 = 2;
pub const FRAME_DURATION_MS: u32 = 10;
/// 10ms 帧的样本数: 48000 * 2 * 0.01 = 960
pub const SAMPLES_PER_FRAME: usize = (SAMPLE_RATE as usize) * (CHANNELS as usize) / 100;

/// 解码后的 PCM 帧
#[derive(Debug, Clone)]
pub struct PcmFrame {
    pub samples: Vec<i16>,
    pub timestamp: u64,
}

/// Opus 解码器，48kHz stereo，10ms 帧长
pub struct Decoder {
    decoder: OpusDecoder,
}

impl Decoder {
    /// 创建新的 Opus 解码器
    pub fn new() -> Result<Self, opus::Error> {
        let decoder = OpusDecoder::new(SAMPLE_RATE, Channels::Stereo)?;
        Ok(Decoder { decoder })
    }

    /// 解码一帧 Opus 数据
    ///
    /// - `data`: Opus 编码数据，空切片表示前向纠错（PLC 补帧）
    /// - `fec`: 是否启用 PLC（丢包补偿时设为 true）
    pub fn decode(&mut self, data: &[u8], fec: bool) -> Result<PcmFrame, opus::Error> {
        let mut pcm_buf = vec![0i16; SAMPLES_PER_FRAME];

        if data.is_empty() {
            // 丢包或需要 PLC 时，使用 Opus 内置的丢包隐藏
            self.decoder.decode(&[], &mut pcm_buf, fec)?;
        } else {
            self.decoder.decode(&data, &mut pcm_buf, fec)?;
        }

        Ok(PcmFrame {
            samples: pcm_buf,
            timestamp: 0, // 由调用方根据需要设置
        })
    }

    /// 解码并设定时间戳
    pub fn decode_with_ts(
        &mut self,
        data: &[u8],
        fec: bool,
        timestamp: u64,
    ) -> Result<PcmFrame, opus::Error> {
        let mut frame = self.decode(data, fec)?;
        frame.timestamp = timestamp;
        Ok(frame)
    }

    /// 重置解码器状态，用于音轨切换或严重丢包后
    pub fn reset(&mut self) -> Result<(), opus::Error> {
        self.decoder.reset_state()
    }
}
