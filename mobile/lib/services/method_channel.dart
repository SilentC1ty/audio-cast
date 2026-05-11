import 'package:flutter/services.dart';

class AudioService {
  static const _channel = MethodChannel('com.audiocast/audio');

  Future<void> requestPermission() async {
    try {
      await _channel.invokeMethod('requestCapturePermission');
    } on PlatformException catch (e) {
      throw Exception('权限请求失败: ${e.message}');
    }
  }

  Future<void> startStreaming(String host, int tcpPort) async {
    try {
      await _channel.invokeMethod('startStreaming', {
        'host': host,
        'port': tcpPort,
        'deviceName': '',
      });
    } on PlatformException catch (e) {
      throw Exception('启动失败: ${e.message}');
    }
  }

  Future<void> stopStreaming() async {
    try {
      await _channel.invokeMethod('stopStreaming');
    } on PlatformException catch (e) {
      throw Exception('停止失败: ${e.message}');
    }
  }

  Future<Map<String, dynamic>> getStatistics() async {
    try {
      final result = await _channel.invokeMethod<Map<Object?, Object?>>('getStatistics');
      return result?.map((k, v) => MapEntry(k.toString(), v)) ?? {};
    } on PlatformException {
      return {};
    }
  }
}
