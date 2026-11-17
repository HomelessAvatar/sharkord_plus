import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/theme.dart';

void main() {
  runApp(const ProviderScope(child: SharkordApp()));
}

class SharkordApp extends StatelessWidget {
  const SharkordApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Sharkord',
      debugShowCheckedModeBanner: false,
      theme: SharkordTheme.darkTheme,
      themeMode: ThemeMode.dark,
      home: const Scaffold(
        body: Center(
          child: Text('Sharkord Mobile'),
        ),
      ),
    );
  }
}
