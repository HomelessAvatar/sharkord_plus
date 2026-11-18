import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/theme.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/screens/login_screen.dart';
import 'package:sharkord_mobile/screens/workspace_screen.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final container = ProviderContainer();
  await container.read(authProvider.notifier).tryRestoreSession();

  runApp(
    UncontrolledProviderScope(
      container: container,
      child: const SharkordApp(),
    ),
  );
}

/// Root widget for the Sharkord mobile application.
class SharkordApp extends ConsumerWidget {
  const SharkordApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isAuthenticated = ref.watch(isAuthenticatedProvider);

    return MaterialApp(
      title: 'Sharkord Plus',
      debugShowCheckedModeBanner: false,
      theme: SharkordTheme.darkTheme,
      themeMode: ThemeMode.dark,
      home: isAuthenticated ? const WorkspaceScreen() : const LoginScreen(),
    );
  }
}
