#pragma once

#include <cstdint>
#include <atomic>

class UdpSender {
public:
    UdpSender(const char* host, int port);
    ~UdpSender();

    bool send(const uint8_t* data, int len, uint32_t seq, uint64_t timestamp);
    bool is_valid() const { return sock_ >= 0; }
    uint64_t packets_sent() const { return packets_sent_.load(); }

private:
    int sock_;
    struct sockaddr_in dest_;
    std::atomic<uint64_t> packets_sent_{0};
};
