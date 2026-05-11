package com.audiocast.mobile

import android.util.Log
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.net.InetSocketAddress
import java.net.Socket
import org.json.JSONObject

class TcpHandshakeClient(
    private val host: String,
    private val port: Int = 19090
) {
    data class HandshakeResult(
        val udpPort: Int,
        val bufferSize: Int,
        val token: String
    )

    fun connect(): HandshakeResult {
        val socket = Socket()
        try {
            socket.connect(InetSocketAddress(host, port), 5000)
            Log.d(TAG, "TCP connected to $host:$port")

            val request = JSONObject().apply {
                put("action", "start")
                put("sampleRate", 48000)
                put("channels", 2)
            }

            val writer = OutputStreamWriter(socket.getOutputStream())
            writer.write(request.toString() + "\n")
            writer.flush()

            val reader = BufferedReader(InputStreamReader(socket.getInputStream()))
            val response = reader.readLine()
            Log.d(TAG, "Handshake response: $response")

            val json = JSONObject(response)
            return HandshakeResult(
                udpPort = json.getInt("udpPort"),
                bufferSize = json.optInt("bufferSize", 80),
                token = json.getString("token")
            )
        } catch (e: Exception) {
            Log.e(TAG, "TCP handshake failed: ${e.message}")
            throw e
        } finally {
            try {
                socket.close()
            } catch (_: Exception) {}
        }
    }

    companion object {
        private const val TAG = "AudioCast.network"
    }
}
