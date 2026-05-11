import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/device.dart';
import '../services/method_channel.dart';
import '../services/mdns_service.dart';

enum ConnectionState {
  idle,
  waiting,
  connected,
  streaming,
  disconnected,
}

class AudioState {
  final ConnectionState connection;
  final List<DiscoveredDevice> devices;
  final String? deviceName;
  final int latency;
  final double packetLoss;
  final int packetsSent;

  const AudioState({
    this.connection = ConnectionState.idle,
    this.devices = const [],
    this.deviceName,
    this.latency = 0,
    this.packetLoss = 0.0,
    this.packetsSent = 0,
  });

  AudioState copyWith({
    ConnectionState? connection,
    List<DiscoveredDevice>? devices,
    String? deviceName,
    int? latency,
    double? packetLoss,
    int? packetsSent,
  }) {
    return AudioState(
      connection: connection ?? this.connection,
      devices: devices ?? this.devices,
      deviceName: deviceName ?? this.deviceName,
      latency: latency ?? this.latency,
      packetLoss: packetLoss ?? this.packetLoss,
      packetsSent: packetsSent ?? this.packetsSent,
    );
  }
}

class AudioStore extends StateNotifier<AudioState> {
  final AudioService _audioService = AudioService();
  final MDnsDiscovery _mdns = MDnsDiscovery();
  Timer? _statsTimer;
  Timer? _discoveryTimer;
  bool _disposed = false;

  AudioStore() : super(const AudioState());

  bool get mounted => !_disposed;

  void init() {
    _startDiscovery();
  }

  void _startDiscovery() {
    state = state.copyWith(connection: ConnectionState.waiting);
    _discoveryTimer?.cancel();
    _discoveryTimer = Timer(const Duration(seconds: 5), () {
      if (state.devices.isEmpty && mounted) {
        state = state.copyWith(connection: ConnectionState.idle);
      }
    });
    _mdns.startDiscovery((devices) {
      _discoveryTimer?.cancel();
      state = state.copyWith(devices: devices);
      if (devices.isEmpty) {
        state = state.copyWith(connection: ConnectionState.idle);
      }
    });
  }

  void refreshDevices() {
    _mdns.stopDiscovery();
    _startDiscovery();
  }

  Future<void> connectToDevice(DiscoveredDevice device) async {
    state = state.copyWith(
      connection: ConnectionState.connected,
      deviceName: device.name,
    );

    try {
      await _audioService.requestPermission();
      await _audioService.startStreaming(device.ip, device.port);
      state = state.copyWith(connection: ConnectionState.streaming);
      _startStatsPolling();
    } catch (e) {
      state = state.copyWith(connection: ConnectionState.disconnected);
    }
  }

  Future<void> disconnect() async {
    _stopStatsPolling();
    try {
      await _audioService.stopStreaming();
    } catch (_) {}
    state = state.copyWith(
      connection: ConnectionState.idle,
      deviceName: null,
      latency: 0,
      packetLoss: 0.0,
      packetsSent: 0,
    );
  }

  void _startStatsPolling() {
    _statsTimer?.cancel();
    _statsTimer = Timer.periodic(const Duration(seconds: 2), (_) async {
      final stats = await _audioService.getStatistics();
      if (stats.isNotEmpty) {
        state = state.copyWith(
          latency: stats['latency'] as int? ?? 0,
          packetLoss: (stats['packetLoss'] as num?)?.toDouble() ?? 0.0,
          packetsSent: stats['packetsSent'] as int? ?? 0,
        );
      }
    });
  }

  void _stopStatsPolling() {
    _statsTimer?.cancel();
    _statsTimer = null;
  }

  @override
  void dispose() {
    _disposed = true;
    _stopStatsPolling();
    _discoveryTimer?.cancel();
    _mdns.dispose();
    super.dispose();
  }
}

final audioStoreProvider = StateNotifierProvider<AudioStore, AudioState>((ref) {
  return AudioStore();
});
