import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sharkord_mobile/core/theme/theme.dart';
import 'package:sharkord_mobile/main.dart';
import 'package:sharkord_mobile/models/channel.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/message_provider.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';
import 'package:sharkord_mobile/screens/login_screen.dart';
import 'package:sharkord_mobile/screens/workspace_screen.dart';
import 'package:sharkord_mobile/widgets/channel_drawer.dart';
import 'package:sharkord_mobile/widgets/chat_canvas.dart';
import 'package:sharkord_mobile/widgets/member_drawer.dart';
import 'package:sharkord_mobile/widgets/voice_bar.dart';
import 'package:sharkord_mobile/widgets/voice_stage_sheet.dart';

Widget createTestApp({
  Widget? home,
  List<Override> overrides = const [],
}) {
  return ProviderScope(
    overrides: overrides,
    child: MaterialApp(
      theme: SharkordTheme.darkTheme,
      home: home ?? const SharkordApp(),
    ),
  );
}

void main() {
  const testUser = User(
    id: '1',
    username: 'shark_tester',
    displayName: 'Shark Tester',
    avatarUrl: null,
    isOnline: true,
  );

  const testChannels = [
    Channel(
      id: '1',
      name: 'general',
      category: 'Text Channels',
      isVoice: false,
    ),
    Channel(
      id: '2',
      name: 'random',
      category: 'Text Channels',
      isVoice: false,
    ),
    Channel(
      id: '3',
      name: 'Voice Lounge',
      category: 'Voice Channels',
      isVoice: true,
      participantCount: 1,
    ),
  ];

  group('LoginScreen Widget Tests', () {
    testWidgets('renders login form fields and submit button', (tester) async {
      await tester.pumpWidget(createTestApp(home: const LoginScreen()));

      expect(find.textContaining('Sharkord'), findsWidgets);
      expect(find.byKey(const Key('login_server_url')), findsOneWidget);
      expect(find.byKey(const Key('login_identity')), findsOneWidget);
      expect(find.byKey(const Key('login_password')), findsOneWidget);
      expect(find.byKey(const Key('login_button')), findsOneWidget);
    });

    testWidgets('validates required fields on empty submit', (tester) async {
      await tester.pumpWidget(createTestApp(home: const LoginScreen()));

      // Tap login without entering fields
      await tester.tap(find.byKey(const Key('login_button')));
      await tester.pumpAndSettle();

      expect(find.textContaining('required', findRichText: true), findsWidgets);
    });

    testWidgets('successful login sets auth session', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const LoginScreen(),
          ),
        ),
      );

      await tester.enterText(
        find.byKey(const Key('login_server_url')),
        'http://localhost:3000',
      );
      await tester.enterText(
        find.byKey(const Key('login_identity')),
        'admin',
      );
      await tester.enterText(
        find.byKey(const Key('login_password')),
        'password123',
      );

      await tester.tap(find.byKey(const Key('login_button')));
      await tester.pumpAndSettle();

      final authState = container.read(authProvider);
      expect(authState.isAuthenticated, isTrue);
      expect(authState.user?.username, 'admin');
    });
  });

  group('WorkspaceScreen Navigation & Chat Feed Tests', () {
    testWidgets('renders app bar with active channel title and buttons', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      container.read(authProvider.notifier).setSession(
            user: testUser,
            token: 'valid_token',
            serverUrl: 'http://localhost:3000',
          );
      container.read(channelProvider.notifier).setChannels(testChannels);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const WorkspaceScreen(),
          ),
        ),
      );

      expect(find.text('general'), findsOneWidget);
      expect(find.byKey(const Key('open_drawer_button')), findsOneWidget);
      expect(find.byKey(const Key('open_members_button')), findsOneWidget);
      expect(find.byKey(const Key('check_update_button')), findsOneWidget);
    });

    testWidgets('left drawer opens and allows channel switching', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      container.read(authProvider.notifier).setSession(
            user: testUser,
            token: 'valid_token',
            serverUrl: 'http://localhost:3000',
          );
      container.read(channelProvider.notifier).setChannels(testChannels);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const WorkspaceScreen(),
          ),
        ),
      );

      // Open drawer
      await tester.tap(find.byKey(const Key('open_drawer_button')));
      await tester.pumpAndSettle();

      expect(find.byType(ChannelDrawer), findsOneWidget);
      expect(find.text('general'), findsWidgets);
      expect(find.text('random'), findsOneWidget);
      expect(find.text('Voice Lounge'), findsOneWidget);

      // Tap random channel
      await tester.tap(find.text('random'));
      await tester.pumpAndSettle();

      // Channel is switched
      expect(container.read(channelProvider).selectedChannelId, '2');
    });

    testWidgets('right drawer opens and shows members roster', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      container.read(authProvider.notifier).setSession(
            user: testUser,
            token: 'valid_token',
            serverUrl: 'http://localhost:3000',
          );
      container.read(channelProvider.notifier).setChannels(testChannels);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const WorkspaceScreen(),
          ),
        ),
      );

      // Open members drawer
      await tester.tap(find.byKey(const Key('open_members_button')));
      await tester.pumpAndSettle();

      expect(find.byType(MemberDrawer), findsOneWidget);
      expect(find.textContaining('ONLINE', findRichText: true), findsOneWidget);
      expect(find.textContaining('OFFLINE', findRichText: true), findsOneWidget);
    });

    testWidgets('message composer sends new message to chat feed', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      container.read(authProvider.notifier).setSession(
            user: testUser,
            token: 'valid_token',
            serverUrl: 'http://localhost:3000',
          );
      container.read(channelProvider.notifier).setChannels(testChannels);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const WorkspaceScreen(),
          ),
        ),
      );

      expect(find.byType(ChatCanvas), findsOneWidget);
      expect(find.byKey(const Key('chat_input_field')), findsOneWidget);

      await tester.enterText(
        find.byKey(const Key('chat_input_field')),
        'Hello Sharkord community!',
      );
      await tester.tap(find.byKey(const Key('chat_send_button')));
      await tester.pumpAndSettle();

      final messages = container.read(messageProvider).getMessages('1');
      expect(messages.length, 1);
      expect(messages.first.content, 'Hello Sharkord community!');
      expect(find.text('Hello Sharkord community!'), findsOneWidget);
    });
  });

  group('VoiceBar & VoiceStageSheet Tests', () {
    testWidgets('VoiceBar hidden when disconnected, visible when connected', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      container.read(authProvider.notifier).setSession(
            user: testUser,
            token: 'valid_token',
            serverUrl: 'http://localhost:3000',
          );
      container.read(channelProvider.notifier).setChannels(testChannels);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const WorkspaceScreen(),
          ),
        ),
      );

      // Disconnected: VoiceBar should not be present
      expect(find.byType(VoiceBar), findsNothing);

      // Join voice
      container.read(voiceProvider.notifier).joinVoice(
            channelId: '3',
            channelName: 'Voice Lounge',
            currentUser: testUser,
          );
      await tester.pumpAndSettle();

      // Connected: VoiceBar should be present
      expect(find.byType(VoiceBar), findsOneWidget);
      expect(find.text('Voice Lounge'), findsOneWidget);
      expect(find.textContaining('24ms'), findsOneWidget);
    });

    testWidgets('VoiceBar mute and deafen toggles work properly', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      container.read(authProvider.notifier).setSession(
            user: testUser,
            token: 'valid_token',
            serverUrl: 'http://localhost:3000',
          );
      container.read(channelProvider.notifier).setChannels(testChannels);
      container.read(voiceProvider.notifier).joinVoice(
            channelId: '3',
            channelName: 'Voice Lounge',
            currentUser: testUser,
          );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const WorkspaceScreen(),
          ),
        ),
      );

      expect(container.read(voiceProvider).isMuted, isFalse);

      // Tap mute
      await tester.tap(find.byKey(const Key('voice_mute_button')));
      await tester.pumpAndSettle();
      expect(container.read(voiceProvider).isMuted, isTrue);

      // Tap deafen
      await tester.tap(find.byKey(const Key('voice_deafen_button')));
      await tester.pumpAndSettle();
      expect(container.read(voiceProvider).isDeafened, isTrue);

      // Tap disconnect
      await tester.tap(find.byKey(const Key('voice_disconnect_button')));
      await tester.pumpAndSettle();
      expect(container.read(voiceProvider).isConnected, isFalse);
      expect(find.byType(VoiceBar), findsNothing);
    });

    testWidgets('tapping VoiceBar opens VoiceStageSheet modal', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      container.read(authProvider.notifier).setSession(
            user: testUser,
            token: 'valid_token',
            serverUrl: 'http://localhost:3000',
          );
      container.read(channelProvider.notifier).setChannels(testChannels);
      container.read(voiceProvider.notifier).joinVoice(
            channelId: '3',
            channelName: 'Voice Lounge',
            currentUser: testUser,
          );
      container.read(voiceProvider.notifier).setSpeaking(testUser.id, true);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            theme: SharkordTheme.darkTheme,
            home: const WorkspaceScreen(),
          ),
        ),
      );

      // Tap voice bar body to open stage sheet
      await tester.tap(find.byKey(const Key('voice_bar_tap_target')));
      await tester.pumpAndSettle();

      expect(find.byType(VoiceStageSheet), findsOneWidget);
      expect(find.text('Shark Tester'), findsOneWidget);
      expect(find.byKey(const Key('stage_disconnect_button')), findsOneWidget);
    });
  });

  group('SharkordApp Root Navigation Test', () {
    testWidgets('shows LoginScreen when unauthenticated and WorkspaceScreen when authenticated', (tester) async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const SharkordApp(),
        ),
      );

      // Default state is unauthenticated -> LoginScreen
      expect(find.byType(LoginScreen), findsOneWidget);
      expect(find.byType(WorkspaceScreen), findsNothing);

      // Set authenticated session
      container.read(authProvider.notifier).setSession(
            user: testUser,
            token: 'sample_token',
            serverUrl: 'http://localhost:3000',
          );
      container.read(channelProvider.notifier).setChannels(testChannels);

      await tester.pumpAndSettle();

      expect(find.byType(WorkspaceScreen), findsOneWidget);
      expect(find.byType(LoginScreen), findsNothing);
    });
  });
}
