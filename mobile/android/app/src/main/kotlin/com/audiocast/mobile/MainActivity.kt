package com.audiocast.mobile

import android.app.Activity
import android.content.Intent
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.util.Log
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

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
                    val port = call.argument<Int>("port") ?: 9999
                    startAudioCapture(host, port)
                    result.success(true)
                }
                "stopStreaming" -> {
                    stopAudioCapture()
                    result.success(true)
                }
                else -> result.notImplemented()
            }
        }
    }

    private fun requestCapturePermission() {
        val manager = getSystemService(MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        val intent = manager.createScreenCaptureIntent()
        // 如果已经有过授权，直接启动服务
        if (pendingResultCode != -1 && pendingIntent != null) {
            return
        }
        startActivityForResult(intent, REQUEST_CODE_CAPTURE)
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == REQUEST_CODE_CAPTURE) {
            if (resultCode == Activity.RESULT_OK && data != null) {
                pendingResultCode = resultCode
                pendingIntent = data
                Log.d("AudioCast", "MediaProjection permission granted")
            }
        }
    }

    private fun startAudioCapture(host: String, port: Int) {
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
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
    }

    private fun stopAudioCapture() {
        val intent = Intent(this, AudioCaptureService::class.java).apply {
            action = "STOP_CAPTURE"
        }
        startService(intent)
    }
}
