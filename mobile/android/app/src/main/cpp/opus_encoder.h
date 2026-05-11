#pragma once

#include <cstdint>

class AudioOpusEncoder {
public:
    AudioOpusEncoder();
    ~AudioOpusEncoder();

    int encode(const int16_t* pcm, int frameSize, uint8_t* output, int maxOutput);
    bool is_valid() const { return enc_ != nullptr; }

private:
    void* enc_;  // OpusEncoder*
};
