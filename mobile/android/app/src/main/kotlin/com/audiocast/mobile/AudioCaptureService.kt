package com.audiocast.mobile

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.IBinder
import android.util.Log
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress

class AudioCaptureService : Service() {

    companion object {
        const val CHANNEL_ID = "audiocast_audio"
        const val SAMPLE_RATE = 48000
        const val CHANNELS = 2
        const val BUFFER_SIZE = 4096
        var targetPort: Int = 0
        var targetHost: String = ""
        var isRunning = false
    }

    private var audioRecord: AudioRecord? = null
    private var udpSocket: DatagramSocket? = null
    private var captureThread: Thread? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        startForeground(1, createNotification())
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            "START_CAPTURE" -> {
                val code = intent.getIntExtra("resultCode", -1)
                val data = intent.getParcelableExtra<Intent>("data")
                if (code != -1 && data != null) {
                    startCapture(code, data)
                }
            }
            "STOP_CAPTURE" -> stopCapture()
        }
        return START_STICKY
    }

    private fun startCapture(resultCode: Int, data: Intent) {
        val projectionManager = getSystemService(MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        val mediaProjection = projectionManager.getMediaProjection(resultCode, data)

        val audioFormat = AudioFormat.Builder()
            .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
            .setSampleRate(SAMPLE_RATE)
            .setChannelMask(AudioFormat.CHANNEL_IN_STEREO)
            .build()

        val playbackConfig = android.media.PlaybackCaptureConfiguration.Builder(
            mediaProjection
        )
            .addMatchingUsage(android.media.AudioAttributes.USAGE_MEDIA)
            .addMatchingUsage(android.media.AudioAttributes.USAGE_GAME)
            .build()

        val audioRecord = AudioRecord.Builder()
            .setAudioPlaybackCaptureConfig(playbackConfig)
            .setAudioFormat(audioFormat)
            .setBufferSizeInBytes(BUFFER_SIZE)
            .build()

        this.audioRecord = audioRecord
        isRunning = true
        audioRecord.startRecording()

        udpSocket = DatagramSocket()
        val targetAddress = InetAddress.getByName(targetHost)

        captureThread = Thread {
            val buffer = ByteArray(BUFFER_SIZE)
            while (isRunning) {
                val bytesRead = audioRecord.read(buffer, 0, buffer.size)
                if (bytesRead > 0) {
                    try {
                        val packet = DatagramPacket(buffer, bytesRead, targetAddress, targetPort)
                        udpSocket?.send(packet)
                    } catch (e: Exception) {
                        Log.e("AudioCapture", "UDP send error", e)
                    }
                }
            }
        }.apply { start() }
    }

    private fun stopCapture() {
        isRunning = false
        captureThread?.join(1000)
        audioRecord?.stop()
        audioRecord?.release()
        audioRecord = null
        udpSocket?.close()
        udpSocket = null
        stopSelf()
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "AudioCast 音频流转",
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = "音频正在转发到桌面端"
        }
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(channel)
    }

    private fun createNotification(): Notification {
        return Notification.Builder(this, CHANNEL_ID)
            .setContentTitle("AudioCast")
            .setContentText("音频正在转发到桌面端")
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setOngoing(true)
            .build()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        stopCapture()
        super.onDestroy()
    }
}
