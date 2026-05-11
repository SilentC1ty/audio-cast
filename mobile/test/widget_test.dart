import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mobile/main.dart';
import 'package:mobile/pages/home_page.dart';

void main() {
  testWidgets('HomePage shows disconnected state initially', (tester) async {
    await tester.pumpWidget(const ProviderScope(child: AudioCastApp()));

    expect(find.text('未连接'), findsOneWidget);
    expect(find.text('可用设备'), findsOneWidget);
    expect(find.text('未发现设备，请确保桌面端已启动'), findsOneWidget);
  });
}
