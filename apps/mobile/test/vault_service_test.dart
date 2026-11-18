import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:sharkord_mobile/core/storage/vault_service.dart';
import 'package:sharkord_mobile/core/theme/theme.dart';
import 'package:sharkord_mobile/models/channel.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/server_provider.dart';
import 'package:sharkord_mobile/widgets/channel_drawer.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('VaultService Tests', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    test('VaultService persists and retrieves server session without prompt', () async {
      SharedPreferences.setMockInitialValues({});
      final vault = VaultService();

      final testServer = SavedServerSession(
        serverId: 'srv_alpha',
        serverName: 'Sharkord Plus HQ',
        serverUrl: 'https://hq.sharkord.com',
        token: 'persisted_jwt_token_999',
        userId: '42',
        username: 'alice',
        displayName: 'Alice',
      );

      await vault.saveServer(testServer, setAsActive: true);

      final active = await vault.getActiveServer();
      expect(active, isNotNull);
      expect(active!.serverId, 'srv_alpha');
      expect(active.token, 'persisted_jwt_token_999');

      final all = await vault.getSavedServers();
      expect(all.length, 1);
    });

    test('VaultService multi-server switching and deletion', () async {
      final vault = VaultService();

      final server1 = SavedServerSession(
        serverId: 'srv_1',
        serverName: 'Server One',
        serverUrl: 'https://one.sharkord.com',
        token: 'token_1',
        userId: '1',
        username: 'user1',
        displayName: 'User One',
      );

      final server2 = SavedServerSession(
        serverId: 'srv_2',
        serverName: 'Server Two',
        serverUrl: 'https://two.sharkord.com',
        token: 'token_2',
        userId: '2',
        username: 'user2',
        displayName: 'User Two',
      );

      await vault.saveServer(server1, setAsActive: true);
      await vault.saveServer(server2, setAsActive: false);

      var all = await vault.getSavedServers();
      expect(all.length, 2);

      var active = await vault.getActiveServer();
      expect(active?.serverId, 'srv_1');

      await vault.setActiveServerId('srv_2');
      active = await vault.getActiveServer();
      expect(active?.serverId, 'srv_2');

      await vault.removeServer('srv_1');
      all = await vault.getSavedServers();
      expect(all.length, 1);
      expect(all.first.serverId, 'srv_2');

      await vault.clearAll();
      all = await vault.getSavedServers();
      expect(all, isEmpty);
      active = await vault.getActiveServer();
      expect(active, isNull);
    });
  });

  group('AuthNotifier Persistent Session Restore Tests', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    test('tryRestoreSession restores session and updates AuthState and ServerDetails', () async {
      final vault = VaultService();
      final testServer = SavedServerSession(
        serverId: 'srv_persisted',
        serverName: 'Persisted Community',
        serverUrl: 'https://community.sharkord.com',
        token: 'valid_vault_token_123',
        userId: '101',
        username: 'restored_user',
        displayName: 'Restored User',
      );
      await vault.saveServer(testServer, setAsActive: true);

      final container = ProviderContainer(
        overrides: [
          vaultServiceProvider.overrideWithValue(vault),
        ],
      );
      addTearDown(container.dispose);

      final restored = await container.read(authProvider.notifier).tryRestoreSession();
      expect(restored, isTrue);

      final authState = container.read(authProvider);
      expect(authState.isAuthenticated, isTrue);
      expect(authState.token, 'valid_vault_token_123');
      expect(authState.serverUrl, 'https://community.sharkord.com');
      expect(authState.user?.id, '101');
      expect(authState.user?.username, 'restored_user');

      final serverDetails = container.read(serverProvider);
      expect(serverDetails.serverId, 'srv_persisted');
      expect(serverDetails.serverName, 'Persisted Community');
    });

    test('tryRestoreSession returns false when vault is empty', () async {
      final vault = VaultService();
      final container = ProviderContainer(
        overrides: [
          vaultServiceProvider.overrideWithValue(vault),
        ],
      );
      addTearDown(container.dispose);

      final restored = await container.read(authProvider.notifier).tryRestoreSession();
      expect(restored, isFalse);

      final authState = container.read(authProvider);
      expect(authState.isAuthenticated, isFalse);
    });

    test('switchServer switches active server session and updates state', () async {
      final vault = VaultService();
      final server1 = SavedServerSession(
        serverId: 'srv_1',
        serverName: 'Server One',
        serverUrl: 'https://one.sharkord.com',
        token: 'token_1',
        userId: '1',
        username: 'alice',
        displayName: 'Alice',
      );
      final server2 = SavedServerSession(
        serverId: 'srv_2',
        serverName: 'Server Two',
        serverUrl: 'https://two.sharkord.com',
        token: 'token_2',
        userId: '2',
        username: 'bob',
        displayName: 'Bob',
      );
      await vault.saveServer(server1, setAsActive: true);
      await vault.saveServer(server2, setAsActive: false);

      final container = ProviderContainer(
        overrides: [
          vaultServiceProvider.overrideWithValue(vault),
        ],
      );
      addTearDown(container.dispose);

      await container.read(authProvider.notifier).tryRestoreSession();
      expect(container.read(authProvider).user?.username, 'alice');

      await container.read(authProvider.notifier).switchServer(server2);
      expect(container.read(authProvider).user?.username, 'bob');
      expect(container.read(authProvider).serverUrl, 'https://two.sharkord.com');
      expect(container.read(serverProvider).serverId, 'srv_2');

      final activeInVault = await vault.getActiveServer();
      expect(activeInVault?.serverId, 'srv_2');
    });

    test('logout with removeFromVault removes active server', () async {
      final vault = VaultService();
      final server1 = SavedServerSession(
        serverId: 'srv_1',
        serverName: 'Server One',
        serverUrl: 'https://one.sharkord.com',
        token: 'token_1',
        userId: '1',
        username: 'alice',
        displayName: 'Alice',
      );
      await vault.saveServer(server1, setAsActive: true);

      final container = ProviderContainer(
        overrides: [
          vaultServiceProvider.overrideWithValue(vault),
        ],
      );
      addTearDown(container.dispose);

      await container.read(authProvider.notifier).tryRestoreSession();
      container.read(authProvider.notifier).logout(removeFromVault: true);

      await Future<void>.delayed(const Duration(milliseconds: 50));
      expect(container.read(authProvider).isAuthenticated, isFalse);
      final saved = await vault.getSavedServers();
      expect(saved, isEmpty);
    });
  });

  group('ChannelDrawer Multi-Server Switcher Widget Tests', () {
    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    testWidgets('renders multiple servers on left rail, supports switching and options modal',
        (tester) async {
      final vault = VaultService();
      final serverA = SavedServerSession(
        serverId: 'srv_a',
        serverName: 'Alpha Server',
        serverUrl: 'https://alpha.sharkord.com',
        token: 'token_a',
        userId: '10',
        username: 'alpha_user',
        displayName: 'Alpha User',
      );
      final serverB = SavedServerSession(
        serverId: 'srv_b',
        serverName: 'Beta Server',
        serverUrl: 'https://beta.sharkord.com',
        token: 'token_b',
        userId: '20',
        username: 'beta_user',
        displayName: 'Beta User',
      );

      await vault.saveServer(serverA, setAsActive: true);
      await vault.saveServer(serverB, setAsActive: false);

      final container = ProviderContainer(
        overrides: [
          vaultServiceProvider.overrideWithValue(vault),
        ],
      );
      addTearDown(container.dispose);

      await container.read(authProvider.notifier).tryRestoreSession();
      container.read(channelProvider.notifier).setChannels(const [
        Channel(id: '1', name: 'general', category: 'Text Channels', isVoice: false),
      ]);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const Scaffold(
              drawer: ChannelDrawer(),
              body: Text('Content'),
            ),
          ),
        ),
      );

      // Open drawer
      final scaffoldState = tester.state<ScaffoldState>(find.byType(Scaffold));
      scaffoldState.openDrawer();
      await tester.pumpAndSettle();

      // Check both servers appear on left rail
      expect(find.byKey(const Key('server_rail_item_srv_a')), findsOneWidget);
      expect(find.byKey(const Key('server_rail_item_srv_b')), findsOneWidget);
      expect(find.byKey(const Key('add_server_rail_button')), findsOneWidget);

      // Tap on Server B to switch
      await tester.tap(find.byKey(const Key('server_rail_item_srv_b')));
      await tester.pumpAndSettle();

      expect(container.read(authProvider).user?.username, 'beta_user');
      expect(container.read(serverProvider).serverId, 'srv_b');

      // Long press Server A to open options modal
      await tester.longPress(find.byKey(const Key('server_rail_item_srv_a')));
      await tester.pumpAndSettle();

      expect(find.text('Alpha Server'), findsOneWidget);
      expect(find.byKey(const Key('delete_server_srv_a')), findsOneWidget);

      // Delete Server A
      await tester.tap(find.byKey(const Key('delete_server_srv_a')));
      await tester.pumpAndSettle();

      final savedAfterDelete = await vault.getSavedServers();
      expect(savedAfterDelete.length, 1);
      expect(savedAfterDelete.first.serverId, 'srv_b');
    });

    testWidgets('Add Server rail button opens dialog with input fields', (tester) async {
      final vault = VaultService();
      final container = ProviderContainer(
        overrides: [
          vaultServiceProvider.overrideWithValue(vault),
        ],
      );
      addTearDown(container.dispose);

      container.read(authProvider.notifier).setSession(
            user: const User(id: '1', username: 'alice', displayName: 'Alice', isOnline: true),
            token: 'test_token',
            serverUrl: 'https://test.sharkord.com',
          );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const Scaffold(
              drawer: ChannelDrawer(),
              body: Text('Content'),
            ),
          ),
        ),
      );

      final scaffoldState = tester.state<ScaffoldState>(find.byType(Scaffold));
      scaffoldState.openDrawer();
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('add_server_rail_button')));
      await tester.pumpAndSettle();

      expect(find.text('Add New Server'), findsOneWidget);
      expect(find.byKey(const Key('add_server_url_field')), findsOneWidget);
      expect(find.byKey(const Key('add_server_identity_field')), findsOneWidget);
      expect(find.byKey(const Key('add_server_password_field')), findsOneWidget);
      expect(find.byKey(const Key('add_server_submit_button')), findsOneWidget);
    });
  });
}
