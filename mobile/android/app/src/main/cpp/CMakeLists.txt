cmake_minimum_required(VERSION 3.18)
project(audiocast_engine)

include(FetchContent)
FetchContent_Declare(
    opus
    GIT_REPOSITORY https://github.com/xiph/opus.git
    GIT_TAG v1.5.2
)
set(OPUS_BUILD_SHARED OFF CACHE BOOL "" FORCE)
set(OPUS_BUILD_TESTING OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(opus)

add_library(audiocast_engine SHARED
    jni_bridge.cpp
    ring_buffer.cpp
    opus_encoder.cpp
    udp_sender.cpp
)

target_include_directories(audiocast_engine PRIVATE
    ${opus_SOURCE_DIR}/include
)

target_link_libraries(audiocast_engine
    log
    opus
)
