import 'dart:async';
import 'package:multicast_dns/multicast_dns.dart';
import '../models/device.dart';

class MDnsDiscovery {
  MDnsClient? _client;
  StreamSubscription<SrvResourceRecord>? _subscription;
  final _devices = <DiscoveredDevice>[];
  final _seenKeys = <String>{};
  void Function(List<DiscoveredDevice>)? _onUpdate;

  List<DiscoveredDevice> get devices => List.unmodifiable(_devices);

  void startDiscovery(void Function(List<DiscoveredDevice>) onUpdate) {
    stopDiscovery();

    _devices.clear();
    _seenKeys.clear();
    _onUpdate = onUpdate;

    _client = MDnsClient();
    _client!.start();

    _subscription = _client!
        .lookup<SrvResourceRecord>('_audiocast._udp.local',
            type: ResourceRecordType.srv)
        .listen(_onServiceFound);
  }

  void _onServiceFound(SrvResourceRecord event) {
    final ip = event.target;
    final key = '${event.name}:$ip:${event.port}';
    if (ip == null || _seenKeys.contains(key)) return;

    _seenKeys.add(key);
    _devices.add(DiscoveredDevice(
      name: event.name,
      ip: ip,
      port: event.port,
    ));
    _onUpdate?.call(List.unmodifiable(_devices));
  }

  void stopDiscovery() {
    _subscription?.cancel();
    _subscription = null;
    _client?.stop();
    _client = null;
    _onUpdate = null;
    _devices.clear();
    _seenKeys.clear();
  }

  void dispose() {
    stopDiscovery();
  }
}
