#include "udp_sender.h"
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <cstring>
#include <android/log.h>

#define LOG_TAG "AudioCast.network"
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

UdpSender::UdpSender(const char* host, int port) {
    sock_ = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock_ < 0) {
        LOGE("socket creation failed");
        return;
    }

    dest_.sin_family = AF_INET;
    dest_.sin_port = htons(static_cast<uint16_t>(port));
    if (inet_pton(AF_INET, host, &dest_.sin_addr) <= 0) {
        LOGE("inet_pton failed for %s", host);
        close(sock_);
        sock_ = -1;
        return;
    }

    LOGD("UDP sender initialized: %s:%d", host, port);
}

UdpSender::~UdpSender() {
    if (sock_ >= 0) {
        close(sock_);
    }
}

bool UdpSender::send(const uint8_t* data, int len,
                      uint32_t seq, uint64_t timestamp) {
    if (sock_ < 0) return false;

    uint8_t packet[1400];
    int offset = 0;

    // Sequence ID (u32 LE)
    packet[offset++] = static_cast<uint8_t>(seq & 0xFF);
    packet[offset++] = static_cast<uint8_t>((seq >> 8) & 0xFF);
    packet[offset++] = static_cast<uint8_t>((seq >> 16) & 0xFF);
    packet[offset++] = static_cast<uint8_t>((seq >> 24) & 0xFF);

    // Timestamp (u64 LE)
    for (int i = 0; i < 8; i++) {
        packet[offset++] = static_cast<uint8_t>((timestamp >> (i * 8)) & 0xFF);
    }

    // Payload length (u16 LE)
    packet[offset++] = static_cast<uint8_t>(len & 0xFF);
    packet[offset++] = static_cast<uint8_t>((len >> 8) & 0xFF);

    // Payload
    if (len > 0) {
        memcpy(packet + offset, data, len);
        offset += len;
    }

    ssize_t sent = sendto(sock_, packet, offset, 0,
                          reinterpret_cast<struct sockaddr*>(&dest_),
                          sizeof(dest_));
    if (sent == offset) {
        packets_sent_.fetch_add(1, std::memory_order_relaxed);
        return true;
    }
    return false;
}
