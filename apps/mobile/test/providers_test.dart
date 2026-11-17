import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:sharkord_mobile/core/update/update_service.dart';
import 'package:sharkord_mobile/models/channel.dart';
import 'package:sharkord_mobile/models/message.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/models/voice_state.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/message_provider.dart';
import 'package:sharkord_mobile/providers/update_provider.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';

void main() {
  group('Dart Data Models', () {
    test('User serialization and equality', () {
      const user = User(
        id: '42',
        username: 'shark_tester',
        displayName: 'Shark Tester',
        avatarUrl: '/avatars/42.png',
        isOnline: true,
      );

      final json = user.toJson();
      expect(json['id'], '42');
      expect(json['username'], 'shark_tester');
      expect(json['displayName'], 'Shark Tester');
      expect(json['avatarUrl'], '/avatars/42.png');
      expect(json['isOnline'], true);

      final deserialized = User.fromJson(json);
      expect(deserialized, equals(user));
      expect(deserialized.numericId, 42);

      final copy = user.copyWith(displayName: 'New Name', isOnline: false);
      expect(copy.displayName, 'New Name');
      expect(copy.isOnline, false);
      expect(copy.id, user.id);
    });

    test('Channel serialization and type helpers', () {
      const textChannel = Channel(
        id: '1',
        name: 'general',
        category: 'Text Channels',
        isVoice: false,
        participantCount: 0,
        activeSpeakerIds: [],
      );

      const voiceChannel = Channel(
        id: '2',
        name: 'Voice Lounge',
        category: 'Voice Channels',
        isVoice: true,
        participantCount: 3,
        activeSpeakerIds: ['10', '20'],
      );

      final json = voiceChannel.toJson();
      expect(json['id'], '2');
      expect(json['isVoice'], true);
      expect(json['participantCount'], 3);
      expect(json['activeSpeakerIds'], ['10', '20']);

      final deserialized = Channel.fromJson(json);
      expect(deserialized, equals(voiceChannel));
      expect(deserialized.numericId, 2);

      final textJson = textChannel.toJson();
      final deserializedText = Channel.fromJson(textJson);
      expect(deserializedText.isVoice, false);
    });

    test('Message serialization and copyWith', () {
      const author = User(
        id: '1',
        username: 'alice',
        displayName: 'Alice',
      );

      final now = DateTime.now();
      final msg = Message(
        id: '100',
        channelId: '1',
        author: author,
        content: 'Testing Riverpod state',
        timestamp: now,
        attachments: const ['https://example.com/image.png'],
        reactions: const ['👍', '🎉'],
      );

      final json = msg.toJson();
      expect(json['id'], '100');
      expect(json['channelId'], '1');
      expect(json['content'], 'Testing Riverpod state');
      expect(json['attachments'], ['https://example.com/image.png']);
      expect(json['reactions'], ['👍', '🎉']);

      final deserialized = Message.fromJson(json);
      expect(deserialized.id, msg.id);
      expect(deserialized.channelId, msg.channelId);
      expect(deserialized.content, msg.content);
      expect(deserialized.numericId, 100);
      expect(deserialized.numericChannelId, 1);
      expect(deserialized.attachments, msg.attachments);
    });

    test('VoiceState serialization and speaker checking', () {
      const user = User(id: '7', username: 'bob', displayName: 'Bob');
      const voice = VoiceState(
        channelId: '3',
        channelName: 'Voice Lounge',
        isConnected: true,
        isMuted: false,
        isDeafened: false,
        pingMs: 32,
        participants: [user],
        activeSpeakerIds: ['7'],
      );

      expect(voice.isSpeaking('7'), true);
      expect(voice.isSpeaking('8'), false);
      expect(voice.numericChannelId, 3);

      final json = voice.toJson();
      final deserialized = VoiceState.fromJson(json);
      expect(deserialized, equals(voice));
    });
  });

  group('AuthNotifier & Providers', () {
    test('initial state is unauthenticated', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final state = container.read(authProvider);
      expect(state.isAuthenticated, false);
      expect(state.user, isNull);
      expect(state.token, isNull);
      expect(state.isLoading, false);
      expect(state.error, isNull);
    });

    test('login validates required inputs', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(authProvider.notifier);

      final res1 = await notifier.login(
        serverUrl: '',
        identity: 'admin',
        password: 'pass',
      );
      expect(res1, false);
      expect(container.read(authProvider).error, contains('Server URL'));

      final res2 = await notifier.login(
        serverUrl: 'http://localhost:3000',
        identity: '',
        password: 'pass',
      );
      expect(res2, false);
      expect(container.read(authProvider).error, contains('Username or email'));

      final res3 = await notifier.login(
        serverUrl: 'http://localhost:3000',
        identity: 'admin',
        password: '',
      );
      expect(res3, false);
      expect(container.read(authProvider).error, contains('Password'));
    });

    test('login succeeds and sets session, then logout clears it', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(authProvider.notifier);

      final success = await notifier.login(
        serverUrl: 'http://localhost:3000',
        identity: 'alice',
        password: 'password123',
      );

      expect(success, true);
      final authState = container.read(authProvider);
      expect(authState.isAuthenticated, true);
      expect(authState.user?.username, 'alice');
      expect(authState.token, isNotNull);
      expect(authState.serverUrl, 'http://localhost:3000');
      expect(container.read(isAuthenticatedProvider), true);
      expect(container.read(currentUserProvider)?.username, 'alice');

      // Logout
      notifier.logout();
      expect(container.read(authProvider).isAuthenticated, false);
      expect(container.read(authProvider).user, isNull);
    });

    test('setSession and clearError work as expected', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(authProvider.notifier);
      const user = User(id: '99', username: 'guest', displayName: 'Guest');

      notifier.setSession(
        user: user,
        token: 'token_abc',
        serverUrl: 'https://sharkord.example.com',
      );

      expect(container.read(authProvider).isAuthenticated, true);
      expect(container.read(authProvider).user?.id, '99');

      notifier.clearError();
      expect(container.read(authProvider).error, isNull);
    });
  });

  group('ChannelNotifier & Providers', () {
    test('initial state and setChannels with auto-selection', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(channelProvider.notifier);
      expect(container.read(channelProvider).channels, isEmpty);

      const channels = [
        Channel(id: '1', name: 'general', isVoice: false),
        Channel(id: '2', name: 'random', isVoice: false),
        Channel(id: '3', name: 'Voice Lounge', isVoice: true),
      ];

      notifier.setChannels(channels);

      final state = container.read(channelProvider);
      expect(state.channels.length, 3);
      expect(state.selectedChannelId, '1');
      expect(state.selectedChannel?.name, 'general');
      expect(container.read(selectedChannelProvider)?.name, 'general');

      expect(container.read(textChannelsProvider).length, 2);
      expect(container.read(voiceChannelsProvider).length, 1);
    });

    test('selectChannel and selectVoiceChannel', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(channelProvider.notifier);
      const channels = [
        Channel(id: '1', name: 'general', isVoice: false),
        Channel(id: '2', name: 'random', isVoice: false),
        Channel(id: '3', name: 'Voice Lounge', isVoice: true),
      ];
      notifier.setChannels(channels);

      notifier.selectChannel('2');
      expect(container.read(selectedChannelProvider)?.id, '2');
      expect(container.read(selectedChannelProvider)?.name, 'random');

      notifier.selectVoiceChannel('3');
      expect(container.read(activeVoiceChannelProvider)?.id, '3');
      expect(container.read(activeVoiceChannelProvider)?.isVoice, true);

      notifier.selectVoiceChannel(null);
      expect(container.read(activeVoiceChannelProvider), isNull);
    });

    test('addChannel, updateChannel, and removeChannel', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(channelProvider.notifier);
      notifier.setChannels([
        const Channel(id: '1', name: 'general', isVoice: false),
      ]);

      // Add channel
      notifier.addChannel(
        const Channel(id: '4', name: 'dev-chat', isVoice: false),
      );
      expect(container.read(channelProvider).channels.length, 2);

      // Update channel
      notifier.updateChannel(
        const Channel(id: '4', name: 'dev-chat-updated', isVoice: false),
      );
      expect(
        container
            .read(channelProvider)
            .channels
            .firstWhere((c) => c.id == '4')
            .name,
        'dev-chat-updated',
      );

      // Remove channel
      notifier.selectChannel('4');
      notifier.removeChannel('4');
      expect(container.read(channelProvider).channels.length, 1);
      expect(container.read(channelProvider).selectedChannelId, isNull);
    });

    test('fetchChannels populates defaults', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(channelProvider.notifier);
      await notifier.fetchChannels();

      expect(container.read(channelProvider).channels.isNotEmpty, true);
      expect(container.read(selectedChannelProvider)?.id, '1');
    });
  });

  group('MessageNotifier & Providers', () {
    test('initial state and setMessages', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(messageProvider.notifier);
      expect(container.read(messageProvider).getMessages('1'), isEmpty);

      const author = User(id: '1', username: 'alice', displayName: 'Alice');
      final msg = Message(
        id: 'm1',
        channelId: '1',
        author: author,
        content: 'First message',
        timestamp: DateTime.now(),
      );

      notifier.setMessages('1', [msg]);
      expect(container.read(messageProvider).getMessages('1').length, 1);
      expect(container.read(channelMessagesProvider('1')).first.content,
          'First message');
    });

    test('sendMessage creates optimistic message and replaces with confirmed',
        () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(messageProvider.notifier);
      const author = User(id: '1', username: 'alice', displayName: 'Alice');

      final sendFuture = notifier.sendMessage(
        channelId: '1',
        content: 'Optimistic UI message',
        author: author,
      );

      // Immediately verify optimistic message exists in state
      final immediateList = container.read(messageProvider).getMessages('1');
      expect(immediateList.length, 1);
      expect(immediateList.first.id.startsWith('temp_'), true);
      expect(immediateList.first.content, 'Optimistic UI message');

      // Await confirmation
      final confirmed = await sendFuture;
      expect(confirmed.id.startsWith('msg_'), true);
      expect(confirmed.content, 'Optimistic UI message');

      // State now holds confirmed message
      final finalList = container.read(messageProvider).getMessages('1');
      expect(finalList.length, 1);
      expect(finalList.first.id, confirmed.id);
    });

    test('receiveMessage, deleteMessage, and clearChannelMessages', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(messageProvider.notifier);
      const author = User(id: '2', username: 'bob', displayName: 'Bob');

      final msg1 = Message(
        id: 'm1',
        channelId: '1',
        author: author,
        content: 'Msg 1',
        timestamp: DateTime.now(),
      );
      final msg2 = Message(
        id: 'm2',
        channelId: '1',
        author: author,
        content: 'Msg 2',
        timestamp: DateTime.now(),
      );

      notifier.receiveMessage(msg1);
      notifier.receiveMessage(msg2);
      expect(container.read(messageProvider).getMessages('1').length, 2);

      // Delete msg1
      notifier.deleteMessage('1', 'm1');
      expect(container.read(messageProvider).getMessages('1').length, 1);
      expect(container.read(messageProvider).getMessages('1').first.id, 'm2');

      // Clear channel
      notifier.clearChannelMessages('1');
      expect(container.read(messageProvider).getMessages('1'), isEmpty);
    });
  });

  group('VoiceNotifier & Providers', () {
    test('joinVoice connects and sets initial state', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(voiceProvider.notifier);
      expect(container.read(voiceProvider).isConnected, false);

      const user = User(id: '1', username: 'alice', displayName: 'Alice');
      notifier.joinVoice(
        channelId: '3',
        channelName: 'Voice Lounge',
        currentUser: user,
      );

      final state = container.read(voiceProvider);
      expect(state.isConnected, true);
      expect(state.channelId, '3');
      expect(state.channelName, 'Voice Lounge');
      expect(state.participants.length, 1);
      expect(state.participants.first.username, 'alice');
      expect(container.read(isVoiceConnectedProvider), true);
      expect(container.read(voiceParticipantsProvider).length, 1);
    });

    test('toggleMute and toggleDeafen lifecycle', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(voiceProvider.notifier);
      notifier.joinVoice(channelId: '3', channelName: 'Voice Lounge');

      // Toggle mute
      expect(container.read(voiceProvider).isMuted, false);
      notifier.toggleMute(true);
      expect(container.read(voiceProvider).isMuted, true);
      notifier.toggleMute(false);
      expect(container.read(voiceProvider).isMuted, false);

      // Deafen automatically mutes
      expect(container.read(voiceProvider).isDeafened, false);
      notifier.toggleDeafen(true);
      expect(container.read(voiceProvider).isDeafened, true);
      expect(container.read(voiceProvider).isMuted, true);

      // Undeafen preserves mute or resets
      notifier.toggleDeafen(false);
      expect(container.read(voiceProvider).isDeafened, false);
    });

    test('speaking indicators and participants management', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(voiceProvider.notifier);
      notifier.joinVoice(channelId: '3', channelName: 'Voice Lounge');

      notifier.setSpeaking('user_10', true);
      expect(container.read(voiceProvider).isSpeaking('user_10'), true);
      expect(container.read(voiceProvider).isSpeaking('user_20'), false);

      notifier.setSpeaking('user_10', false);
      expect(container.read(voiceProvider).isSpeaking('user_10'), false);

      // Add/remove participants
      const user2 = User(id: '2', username: 'bob', displayName: 'Bob');
      notifier.addParticipant(user2);
      expect(container.read(voiceProvider).participants.length, 1);

      notifier.removeParticipant('2');
      expect(container.read(voiceProvider).participants, isEmpty);

      // Ping update
      notifier.updatePing(45);
      expect(container.read(voiceProvider).pingMs, 45);

      // Leave voice
      notifier.leaveVoice();
      expect(container.read(voiceProvider).isConnected, false);
      expect(container.read(voiceProvider).channelId, isNull);
    });
  });

  group('UpdateNotifier & Providers', () {
    test('checkForUpdate detects new version and resets', () async {
      final mockClient = MockClient((request) async {
        return http.Response('''
        {
          "tag_name": "v0.3.0",
          "name": "Sharkord v0.3.0",
          "body": "Major release notes",
          "published_at": "2026-11-17T12:00:00Z",
          "html_url": "https://github.com/HomelessAvatar/sharkord_plus/releases/tag/v0.3.0",
          "assets": [
            {
              "name": "sharkord-mobile-android.apk",
              "size": 25000000,
              "browser_download_url": "https://github.com/HomelessAvatar/sharkord_plus/releases/download/v0.3.0/sharkord-mobile-android.apk"
            }
          ]
        }
        ''', 200);
      });

      final mockService = UpdateService(
        repo: 'test/repo',
        client: mockClient,
      );

      final container = ProviderContainer(
        overrides: [
          updateServiceProvider.overrideWithValue(mockService),
        ],
      );
      addTearDown(container.dispose);

      final notifier = container.read(updateProvider.notifier);
      final result = await notifier.checkForUpdate(
        currentVersionOverride: '0.2.0',
        isAndroidOverride: true,
      );

      expect(result?.hasUpdate, true);
      expect(container.read(hasUpdateProvider), true);
      expect(container.read(updateProvider).hasUpdate, true);
      expect(container.read(updateProvider).updateResult?.latestRelease?.tagName,
          'v0.3.0');

      // Download progress
      notifier.setDownloadProgress(0.5);
      expect(container.read(updateProvider).downloadProgress, 0.5);
      expect(container.read(updateProvider).isDownloading, true);

      notifier.setDownloadProgress(1.0);
      expect(container.read(updateProvider).downloadProgress, 1.0);
      expect(container.read(updateProvider).isDownloading, false);

      // Reset
      notifier.reset();
      expect(container.read(updateProvider).hasUpdate, false);
      expect(container.read(updateProvider).downloadProgress, 0.0);
    });
  });
}
