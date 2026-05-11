#pragma once

#include <cstdint>
#include <cstring>
#include <atomic>

constexpr int kSamplesPerFrame = 960;   // 48000 * 2ch * 0.01s
constexpr int kBufferFrames = 20;        // 200ms 缓冲

struct PCMFrame {
    int16_t samples[kSamplesPerFrame];
    uint64_t timestamp;
};

class RingBuffer {
public:
    RingBuffer() = default;

    bool push(const int16_t* data, int count, uint64_t timestamp) {
        int w = write_pos_.load(std::memory_order_relaxed);
        int r = read_pos_.load(std::memory_order_acquire);
        int next_w = (w + 1) % kBufferFrames;
        if (next_w == r) return false;

        auto& frame = buffer_[w];
        memcpy(frame.samples, data, count * sizeof(int16_t));
        frame.timestamp = timestamp;
        write_pos_.store(next_w, std::memory_order_release);
        return true;
    }

    bool pop(PCMFrame& frame) {
        int r = read_pos_.load(std::memory_order_relaxed);
        int w = write_pos_.load(std::memory_order_acquire);
        if (r == w) return false;

        frame = buffer_[r];
        read_pos_.store((r + 1) % kBufferFrames, std::memory_order_release);
        return true;
    }

    bool empty() const {
        int r = read_pos_.load(std::memory_order_acquire);
        int w = write_pos_.load(std::memory_order_acquire);
        return r == w;
    }

    bool full() const {
        int w = write_pos_.load(std::memory_order_relaxed);
        int r = read_pos_.load(std::memory_order_acquire);
        return (w + 1) % kBufferFrames == r;
    }

private:
    PCMFrame buffer_[kBufferFrames];
    std::atomic<int> read_pos_{0};
    std::atomic<int> write_pos_{0};
};
