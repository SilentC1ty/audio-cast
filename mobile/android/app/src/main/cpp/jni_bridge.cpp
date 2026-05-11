#include <jni.h>
#include <thread>
#include <atomic>
#include <chrono>
#include <android/log.h>

#include "ring_buffer.h"
#include "opus_encoder.h"
#include "udp_sender.h"

#define LOG_TAG "AudioCast.jni"
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// --- Encoded packet queue (SPSC, lock-free) ---

struct EncodedPacket {
    uint8_t data[1400];
    int size;
    uint64_t timestamp;
};

class PacketQueue {
    static constexpr int kCapacity = 60;  // 600ms buffer
    EncodedPacket buffer_[kCapacity];
    std::atomic<int> read_pos_{0};
    std::atomic<int> write_pos_{0};

public:
    bool push(const uint8_t* data, int size, uint64_t timestamp) {
        int w = write_pos_.load(std::memory_order_relaxed);
        int r = read_pos_.load(std::memory_order_acquire);
        int next_w = (w + 1) % kCapacity;
        if (next_w == r) return false;

        auto& pkt = buffer_[w];
        memcpy(pkt.data, data, size);
        pkt.size = size;
        pkt.timestamp = timestamp;
        write_pos_.store(next_w, std::memory_order_release);
        return true;
    }

    bool pop(EncodedPacket& pkt) {
        int r = read_pos_.load(std::memory_order_relaxed);
        int w = write_pos_.load(std::memory_order_acquire);
        if (r == w) return false;

        pkt = buffer_[r];
        read_pos_.store((r + 1) % kCapacity, std::memory_order_release);
        return true;
    }
};

// --- Audio engine ---

class AudioEngine {
public:
    AudioEngine(const char* host, int port)
        : sender_(host, port), running_(true) {
        if (!sender_.is_valid()) {
            running_ = false;
            return;
        }
        encode_thread_ = std::thread(&AudioEngine::encodeLoop, this);
        send_thread_ = std::thread(&AudioEngine::sendLoop, this);
        LOGD("Audio engine started, sending to %s:%d", host, port);
    }

    ~AudioEngine() {
        stop();
    }

    void pushPCM(const int16_t* data, int count, uint64_t timestamp) {
        if (!running_) return;
        int frames = count / kSamplesPerFrame;
        for (int i = 0; i < frames; i++) {
            pcm_buffer_.push(data + i * kSamplesPerFrame, kSamplesPerFrame, timestamp);
        }
        cv_.notify_one();
    }

    void stop() {
        bool expected = true;
        if (!running_.compare_exchange_strong(expected, false)) return;

        cv_.notify_all();
        if (encode_thread_.joinable()) encode_thread_.join();
        if (send_thread_.joinable()) send_thread_.join();
        LOGD("Audio engine stopped");
    }

    bool is_running() const { return running_.load(); }
    uint64_t packets_sent() const { return sender_.packets_sent(); }

private:
    void encodeLoop() {
        PCMFrame frame;
        uint8_t opus_data[4000];

        while (running_) {
            if (pcm_buffer_.pop(frame)) {
                int len = encoder_.encode(frame.samples, kSamplesPerFrame,
                                          opus_data, sizeof(opus_data));
                if (len > 0) {
                    send_buffer_.push(opus_data, len, frame.timestamp);
                }
            } else {
                std::unique_lock<std::mutex> lock(mutex_);
                cv_.wait_for(lock, std::chrono::milliseconds(5));
            }
        }
    }

    void sendLoop() {
        EncodedPacket pkt;
        uint32_t seq = 0;

        while (running_) {
            if (send_buffer_.pop(pkt)) {
                sender_.send(pkt.data, pkt.size, seq++, pkt.timestamp);
            } else {
                std::this_thread::sleep_for(std::chrono::milliseconds(1));
            }
        }
    }

    RingBuffer pcm_buffer_;
    PacketQueue send_buffer_;
    AudioOpusEncoder encoder_;
    UdpSender sender_;
    std::atomic<bool> running_;
    std::thread encode_thread_;
    std::thread send_thread_;
    std::mutex mutex_;
    std::condition_variable cv_;
};

// --- JNI bridge ---

extern "C" {

JNIEXPORT jlong JNICALL
Java_com_audiocast_mobile_AudioEngine_nativeInit(
    JNIEnv* env, jobject /*thiz*/, jstring host, jint port) {
    const char* host_cstr = env->GetStringUTFChars(host, nullptr);
    auto* engine = new AudioEngine(host_cstr, static_cast<int>(port));
    env->ReleaseStringUTFChars(host, host_cstr);

    if (!engine->is_running()) {
        delete engine;
        return 0;
    }
    return reinterpret_cast<jlong>(engine);
}

JNIEXPORT void JNICALL
Java_com_audiocast_mobile_AudioEngine_nativePushPCM(
    JNIEnv* env, jobject /*thiz*/, jlong handle, jshortArray pcm,
    jlong timestamp) {
    if (handle == 0) return;
    auto* engine = reinterpret_cast<AudioEngine*>(handle);

    jsize len = env->GetArrayLength(pcm);
    jshort* elements = env->GetShortArrayElements(pcm, nullptr);
    engine->pushPCM(reinterpret_cast<const int16_t*>(elements),
                    static_cast<int>(len),
                    static_cast<uint64_t>(timestamp));
    env->ReleaseShortArrayElements(pcm, elements, JNI_ABORT);
}

JNIEXPORT jstring JNICALL
Java_com_audiocast_mobile_AudioEngine_nativeGetStats(
    JNIEnv* env, jobject /*thiz*/, jlong handle) {
    if (handle == 0) return env->NewStringUTF("{}");
    auto* engine = reinterpret_cast<AudioEngine*>(handle);

    char json[128];
    snprintf(json, sizeof(json),
             "{\"packetsSent\":%llu}",
             static_cast<unsigned long long>(engine->packets_sent()));
    return env->NewStringUTF(json);
}

JNIEXPORT void JNICALL
Java_com_audiocast_mobile_AudioEngine_nativeStop(
    JNIEnv* /*env*/, jobject /*thiz*/, jlong handle) {
    if (handle == 0) return;
    auto* engine = reinterpret_cast<AudioEngine*>(handle);
    engine->stop();
    delete engine;
}

}  // extern "C"
