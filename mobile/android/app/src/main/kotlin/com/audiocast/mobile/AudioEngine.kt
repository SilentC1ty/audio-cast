package com.audiocast.mobile

import android.util.Log

class AudioEngine(host: String, port: Int) {
    private var nativeHandle: Long = 0
    private val TAG = "AudioCast.audio"

    fun start(): Boolean {
        nativeHandle = nativeInit(host, port)
        if (nativeHandle == 0L) {
            Log.e(TAG, "Failed to init native audio engine")
            return false
        }
        Log.d(TAG, "Native engine started, handle=$nativeHandle")
        return true
    }

    fun pushPCM(pcm: ShortArray, timestamp: Long) {
        if (nativeHandle != 0L) {
            nativePushPCM(nativeHandle, pcm, timestamp)
        }
    }

    fun stop() {
        if (nativeHandle != 0L) {
            nativeStop(nativeHandle)
            nativeHandle = 0
            Log.d(TAG, "Native engine stopped")
        }
    }

    fun getStats(): String {
        return if (nativeHandle != 0L) nativeGetStats(nativeHandle) else "{}"
    }

    private external fun nativeInit(host: String, port: Int): Long
    private external fun nativePushPCM(handle: Long, pcm: ShortArray, timestamp: Long)
    private external fun nativeGetStats(handle: Long): String
    private external fun nativeStop(handle: Long)

    companion object {
        init {
            System.loadLibrary("audiocast_engine")
        }
    }
}
