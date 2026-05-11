package com.audiocast.mobile

import android.app.Activity
import android.content.Intent
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.util.Log
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import org.json.JSONObject

class MainActivity : FlutterActivity() {
    private val CHANNEL = "com.audiocast/audio"
    private val REQUEST_CODE_CAPTURE = 1001

    private var pendingIntent: Intent? = null
    private var pendingResultCode: Int = -1

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL).setMethodCallHandler { call, result ->
            when (call.method) {
                "requestCapturePermission" -> {
                    requestCapturePermission()
                    result.success(true)
                }
                "startStreaming" -> {
                    val host = call.argument<String>("host") ?: ""
                    val port = call.argument<Int>("port") ?: 19090
                    val deviceName = call.argument<String>("deviceName") ?: ""
                    startAudioCapture(host, port, deviceName)
                    result.success(true)
                }
                "stopStreaming" -> {
                    stopAudioCapture()
                    result.success(true)
                }
                "getStatistics" -> {
                    val stats = getStatistics()
                    result.success(stats)
                }
                else -> result.notImplemented()
            }
        }
    }

    private fun requestCapturePermission() {
        if (pendingResultCode != -1 && pendingIntent != null) return
        val manager = getSystemService(MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        startActivityForResult(manager.createScreenCaptureIntent(), REQUEST_CODE_CAPTURE)
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == REQUEST_CODE_CAPTURE) {
            if (resultCode == Activity.RESULT_OK && data != null) {
                pendingResultCode = resultCode
                pendingIntent = data
                Log.d(TAG, "MediaProjection permission granted")
            }
        }
    }

    private fun startAudioCapture(host: String, port: Int, deviceName: String = "") {
        if (pendingResultCode == -1 || pendingIntent == null) {
            requestCapturePermission()
            return
        }

        AudioCaptureService.targetHost = host
        AudioCaptureService.targetPort = port

        val intent = Intent(this, AudioCaptureService::class.java).apply {
            action = "START_CAPTURE"
            putExtra("resultCode", pendingResultCode)
            putExtra("data", pendingIntent)
            putExtra("deviceName", deviceName)
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
    }

    private fun stopAudioCapture() {
        startService(Intent(this, AudioCaptureService::class.java).apply {
            action = "STOP_CAPTURE"
        })
    }

    private fun getStatistics(): Map<String, Any> {
        val stats = mutableMapOf<String, Any>()
        stats["latency"] = 0
        stats["packetLoss"] = 0.0
        return stats
    }

    companion object {
        private const val TAG = "AudioCast.system"
    }
}
