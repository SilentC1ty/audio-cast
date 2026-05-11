package com.audiocast.mobile

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.IBinder
import android.util.Log

class AudioCaptureService : Service() {

    companion object {
        const val CHANNEL_ID = "audiocast_audio"
        const val SAMPLE_RATE = 48000
        const val CHANNELS = 2
        const val BUFFER_SIZE = 4096
        var targetHost: String = ""
        var targetPort: Int = 19090
        var currentDeviceName: String = ""
        var isRunning = false
        private const val TAG = "AudioCast.audio"
    }

    private var audioRecord: AudioRecord? = null
    private var audioEngine: AudioEngine? = null
    private var captureThread: Thread? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            "START_CAPTURE" -> {
                val code = intent.getIntExtra("resultCode", -1)
                val data = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    intent.getParcelableExtra("data", Intent::class.java)
                } else {
                    @Suppress("DEPRECATION")
                    intent.getParcelableExtra("data")
                }
                currentDeviceName = intent.getStringExtra("deviceName") ?: ""
                startForeground(1, createNotification("正在连接 $currentDeviceName..."))
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

        val playbackConfig = android.media.PlaybackCaptureConfiguration.Builder(mediaProjection)
            .addMatchingUsage(android.media.AudioAttributes.USAGE_MEDIA)
            .addMatchingUsage(android.media.AudioAttributes.USAGE_GAME)
            .build()

        val record = AudioRecord.Builder()
            .setAudioPlaybackCaptureConfig(playbackConfig)
            .setAudioFormat(audioFormat)
            .setBufferSizeInBytes(BUFFER_SIZE)
            .build()

        this.audioRecord = record

        // TCP handshake on a background thread
        Thread {
            try {
                val handshake = TcpHandshakeClient(targetHost, 19090).connect()
                Log.d(TAG, "Handshake OK: udpPort=${handshake.udpPort}, token=${handshake.token}")

                val engine = AudioEngine(targetHost, handshake.udpPort)
                if (!engine.start()) {
                    Log.e(TAG, "Failed to start native engine")
                    stopCapture()
                    return@Thread
                }
                this.audioEngine = engine
                isRunning = true

                record.startRecording()

                captureThread = Thread {
                    val buffer = ShortArray(BUFFER_SIZE / 2)
                    while (isRunning) {
                        val samplesRead = record.read(buffer, 0, buffer.size)
                        if (samplesRead > 0) {
                            engine.pushPCM(buffer, System.currentTimeMillis())
                        }
                    }
                }.apply { start() }

                updateNotification("音频流转中 - $currentDeviceName")
                Log.d(TAG, "Capture started, sending to ${targetHost}:${handshake.udpPort}")

            } catch (e: Exception) {
                Log.e(TAG, "Failed to start streaming", e)
                stopCapture()
            }
        }.apply { start() }
    }

    private fun stopCapture() {
        isRunning = false
        captureThread?.join(1000)
        captureThread = null

        try {
            audioRecord?.stop()
        } catch (_: Exception) {}
        audioRecord?.release()
        audioRecord = null

        audioEngine?.stop()
        audioEngine = null

        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
        Log.d(TAG, "Capture stopped")
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

    private fun createNotification(text: String): Notification {
        return Notification.Builder(this, CHANNEL_ID)
            .setContentTitle("AudioCast")
            .setContentText(text)
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setOngoing(true)
            .build()
    }

    private fun updateNotification(text: String) {
        val manager = getSystemService(NotificationManager::class.java)
        manager.notify(1, createNotification(text))
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        stopCapture()
        super.onDestroy()
    }
}
