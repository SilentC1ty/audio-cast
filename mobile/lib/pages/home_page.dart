import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/device.dart';
import '../stores/audio_store.dart';

class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(audioStoreProvider);

    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.surface,
      body: SafeArea(
        child: Column(
          children: [
            const Spacer(flex: 2),
            _buildStatusIndicator(state),
            const Spacer(flex: 1),
            if (state.connection == ConnectionState.streaming)
              _buildNetworkInfo(state),
            const Spacer(flex: 2),
          ],
        ),
      ),
      bottomSheet: _buildDeviceSheet(context, ref, state),
    );
  }

  Widget _buildStatusIndicator(AudioState state) {
    final (icon, label, color) = switch (state.connection) {
      ConnectionState.idle || ConnectionState.disconnected => (
        Icons.wifi_off,
        '未连接',
        Colors.grey,
      ),
      ConnectionState.waiting => (
        Icons.wifi_find,
        '搜索设备中...',
        Colors.amber,
      ),
      ConnectionState.connected => (
        Icons.wifi_password,
        '连接中...',
        Colors.blue,
      ),
      ConnectionState.streaming => (
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
        const SizedBox(height: 12),
        Text(
          label,
          style: TextStyle(
            fontSize: 20,
            fontWeight: FontWeight.w500,
            color: color,
          ),
        ),
        if (state.deviceName != null) ...[
          const SizedBox(height: 4),
          Text(
            state.deviceName!,
            style: const TextStyle(fontSize: 14, color: Colors.grey),
          ),
        ],
      ],
    );
  }

  Widget _buildNetworkInfo(AudioState state) {
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
          _infoItem('延迟', '${state.latency} ms'),
          _infoItem('丢包', '${state.packetLoss.toStringAsFixed(1)} %'),
          _infoItem('已发送', '${state.packetsSent}'),
        ],
      ),
    );
  }

  Widget _infoItem(String label, String value) {
    return Column(
      children: [
        Text(value,
            style: const TextStyle(
                fontSize: 18, fontWeight: FontWeight.bold)),
        const SizedBox(height: 4),
        Text(label,
            style: const TextStyle(fontSize: 12, color: Colors.grey)),
      ],
    );
  }

  Widget _buildDeviceSheet(
      BuildContext context, WidgetRef ref, AudioState state) {
    final isStreaming =
        state.connection == ConnectionState.streaming;
    final isConnecting =
        state.connection == ConnectionState.connected;

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
                onPressed: isConnecting
                    ? null
                    : () => ref
                        .read(audioStoreProvider.notifier)
                        .refreshDevices(),
                icon: const Icon(Icons.refresh, size: 18),
                label: const Text('刷新'),
              ),
            ],
          ),
          const SizedBox(height: 8),
          if (state.devices.isEmpty)
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
            ...state.devices.map((device) => Card(
                  child: ListTile(
                    leading: const Icon(Icons.desktop_windows),
                    title: Text(device.name),
                    subtitle: Text('${device.ip}:${device.port}'),
                    trailing: isStreaming
                        ? IconButton(
                            icon: const Icon(Icons.stop_circle_outlined,
                                color: Colors.red),
                            onPressed: () => ref
                                .read(audioStoreProvider.notifier)
                                .disconnect(),
                          )
                        : const Icon(Icons.chevron_right),
                    onTap: (!isStreaming && !isConnecting)
                        ? () => ref
                            .read(audioStoreProvider.notifier)
                            .connectToDevice(device)
                        : null,
                  ),
                )),
          const SizedBox(height: 8),
        ],
      ),
    );
  }
}
