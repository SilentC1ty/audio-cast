#include "opus_encoder.h"
#include <opus/opus.h>
#include <android/log.h>

#define LOG_TAG "AudioCast.codec"
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

AudioOpusEncoder::AudioOpusEncoder() : enc_(nullptr) {
    int error;
    enc_ = opus_encoder_create(48000, 2, OPUS_APPLICATION_AUDIO, &error);
    if (error != OPUS_OK || !enc_) {
        LOGE("opus_encoder_create failed: %s", opus_strerror(error));
        return;
    }

    opus_encoder_ctl(enc_, OPUS_SET_BITRATE(96000));
    opus_encoder_ctl(enc_, OPUS_SET_VBR(1));
    opus_encoder_ctl(enc_, OPUS_SET_COMPLEXITY(5));
    opus_encoder_ctl(enc_, OPUS_SET_SIGNAL(OPUS_SIGNAL_MUSIC));

    LOGD("Opus encoder created: 48kHz/2ch/96kbps/VBR");
}

AudioOpusEncoder::~AudioOpusEncoder() {
    if (enc_) {
        opus_encoder_destroy(static_cast<OpusEncoder*>(enc_));
    }
}

int AudioOpusEncoder::encode(const int16_t* pcm, int frameSize,
                              uint8_t* output, int maxOutput) {
    if (!enc_) return -1;

    auto* enc = static_cast<OpusEncoder*>(enc_);
    int result = opus_encode(enc, pcm, frameSize / 2, output, maxOutput);
    if (result < 0) {
        LOGE("opus_encode failed: %s", opus_strerror(result));
        return -1;
    }
    return result;
}
