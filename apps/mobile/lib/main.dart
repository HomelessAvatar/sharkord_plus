import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/theme.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/screens/login_screen.dart';
import 'package:sharkord_mobile/screens/workspace_screen.dart';

void main() {
  runApp(const ProviderScope(child: SharkordApp()));
}

/// Root widget for the Sharkord mobile application.
class SharkordApp extends ConsumerWidget {
  const SharkordApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isAuthenticated = ref.watch(isAuthenticatedProvider);

    return MaterialApp(
      title: 'Sharkord',
      debugShowCheckedModeBanner: false,
      theme: SharkordTheme.darkTheme,
      themeMode: ThemeMode.dark,
      home: isAuthenticated ? const WorkspaceScreen() : const LoginScreen(),
    );
  }
}
