import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:multicast_dns/multicast_dns.dart';

const platform = MethodChannel('com.audiocast/audio');

enum StreamStatus { disconnected, searching, streaming }

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const AudioCastApp());
}

class AudioCastApp extends StatelessWidget {
  const AudioCastApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'AudioCast',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.indigo,
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      home: const HomePage(),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  StreamStatus _status = StreamStatus.disconnected;
  List<DiscoveredDevice> _devices = [];
  MDnsClient? _mdnsClient;
  StreamSubscription? _mdnsSubscription;
  String _latency = '--';
  String _packetLoss = '--';

  @override
  void initState() {
    super.initState();
    _requestPermission();
    _startDiscovery();
  }

  Future<void> _requestPermission() async {
    try {
      await platform.invokeMethod('requestCapturePermission');
    } on PlatformException catch (e) {
      debugPrint('Permission error: $e');
    }
  }

  void _startDiscovery() {
    setState(() => _status = StreamStatus.searching);
    _mdnsClient?.stop();
    _mdnsClient = MDnsClient();
    _mdnsClient!.start();

    _mdnsSubscription = _mdnsClient!
        .lookup('_audiocast._tcp', type: ResourceRecordType.srv)
        .listen((event) {
      final ip = event.target;
      if (ip != null) {
        setState(() {
          _devices.add(DiscoveredDevice(
            name: event.name,
            ip: ip,
            port: event.port,
          ));
        });
      }
    });

    // 5秒超时，如果没发现设备则停止搜索状态
    Timer(const Duration(seconds: 5), () {
      if (_devices.isEmpty && mounted) {
        setState(() => _status = StreamStatus.disconnected);
      }
    });
  }

  Future<void> _connectToDevice(DiscoveredDevice device) async {
    setState(() => _status = StreamStatus.streaming);
    try {
      await platform.invokeMethod('startStreaming', {
        'host': device.ip,
        'port': device.port,
      });
    } on PlatformException catch (e) {
      debugPrint('Stream error: $e');
      setState(() => _status = StreamStatus.disconnected);
    }
  }

  Future<void> _disconnect() async {
    try {
      await platform.invokeMethod('stopStreaming');
    } on PlatformException catch (e) {
      debugPrint('Stop error: $e');
    }
    setState(() => _status = StreamStatus.disconnected);
  }

  @override
  void dispose() {
    _mdnsSubscription?.cancel();
    _mdnsClient?.stop();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.surface,
      body: SafeArea(
        child: Column(
          children: [
            const Spacer(flex: 2),
            // 状态指示器
            _buildStatusIndicator(),
            const Spacer(flex: 1),
            // 网络状态
            if (_status == StreamStatus.streaming) _buildNetworkInfo(),
            const Spacer(flex: 2),
          ],
        ),
      ),
      bottomSheet: _buildDeviceSheet(),
    );
  }

  Widget _buildStatusIndicator() {
    final (icon, label, color) = switch (_status) {
      StreamStatus.disconnected => (
        Icons.wifi_off,
        '未连接',
        Colors.grey,
      ),
      StreamStatus.searching => (
        Icons.wifi_find,
        '搜索设备中...',
        Colors.amber,
      ),
      StreamStatus.streaming => (
        Icons.wifi,
        '音频流转中',
        Colors.green,
      ),
    };

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 100,
          height: 100,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: color.withAlpha(40),
          ),
          child: Icon(icon, size: 48, color: color),
        ),
        const SizedBox(height: 20),
        Text(
          label,
          style: TextStyle(
            fontSize: 20,
            fontWeight: FontWeight.w500,
            color: color,
          ),
        ),
      ],
    );
  }

  Widget _buildNetworkInfo() {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 40),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _infoItem('延迟', '$_latency ms'),
          _infoItem('丢包', '$_packetLoss %'),
        ],
      ),
    );
  }

  Widget _infoItem(String label, String value) {
    return Column(
      children: [
        Text(value, style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
        const SizedBox(height: 4),
        Text(label, style: const TextStyle(fontSize: 12, color: Colors.grey)),
      ],
    );
  }

  Widget _buildDeviceSheet() {
    return Container(
      padding: const EdgeInsets.all(20),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                '可用设备',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              TextButton.icon(
                onPressed: _startDiscovery,
                icon: const Icon(Icons.refresh, size: 18),
                label: const Text('刷新'),
              ),
            ],
          ),
          const SizedBox(height: 8),
          if (_devices.isEmpty)
            Container(
              width: double.infinity,
              padding: const EdgeInsets.symmetric(vertical: 24),
              child: const Text(
                '未发现设备，请确保桌面端已启动',
                textAlign: TextAlign.center,
                style: TextStyle(color: Colors.grey),
              ),
            )
          else
            ..._devices.map((device) => Card(
              child: ListTile(
                leading: const Icon(Icons.desktop_windows),
                title: Text(device.name),
                subtitle: Text('${device.ip}:${device.port}'),
                trailing: _status == StreamStatus.streaming
                    ? IconButton(
                        icon: const Icon(Icons.stop_circle_outlined, color: Colors.red),
                        onPressed: _disconnect,
                      )
                    : const Icon(Icons.chevron_right),
                onTap: _status != StreamStatus.streaming
                    ? () => _connectToDevice(device)
                    : null,
              ),
            )),
          const SizedBox(height: 8),
        ],
      ),
    );
  }
}

class DiscoveredDevice {
  final String name;
  final String ip;
  final int port;

  DiscoveredDevice({
    required this.name,
    required this.ip,
    required this.port,
  });
}
