import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sharkord_mobile/models/channel.dart';
import 'package:sharkord_mobile/models/message.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/message_provider.dart';
import 'package:sharkord_mobile/widgets/chat_canvas.dart';

void main() {
  group('Interactive Components & Plugin Protocol Models', () {
    test('PluginCommand serialization and equality', () {
      const cmd = PluginCommand(
        pluginId: 'soundboard',
        name: 'play',
        description: 'Play sound bite into voice channel',
      );

      final json = cmd.toJson();
      expect(json['pluginId'], 'soundboard');
      expect(json['name'], 'play');
      expect(json['description'], 'Play sound bite into voice channel');

      final fromJson = PluginCommand.fromJson(json);
      expect(fromJson, equals(cmd));
    });

    test('PluginButton parsing and serialization', () {
      const button = PluginButton(
        id: 'btn-1',
        label: 'Roll Dice',
        pluginId: 'dice_roller',
        actionName: 'roll',
        style: 'primary',
      );

      final json = button.toJson();
      expect(json['id'], 'btn-1');
      expect(json['label'], 'Roll Dice');
      expect(json['pluginId'], 'dice_roller');
      expect(json['actionName'], 'roll');

      final fromJson = PluginButton.fromJson(json);
      expect(fromJson, equals(button));
    });

    test('PluginPoll parsing and calculation', () {
      const poll = PluginPoll(
        id: 'poll-1',
        question: 'What is your favorite framework?',
        options: [
          PluginPollOption(id: 'flutter', text: 'Flutter', votes: 15, voted: true),
          PluginPollOption(id: 'react', text: 'React', votes: 5, voted: false),
        ],
        pluginId: 'polls',
        actionName: 'vote',
        totalVotes: 20,
      );

      final json = poll.toJson();
      expect(json['question'], 'What is your favorite framework?');
      expect((json['options'] as List).length, 2);

      final fromJson = PluginPoll.fromJson(json);
      expect(fromJson.id, 'poll-1');
      expect(fromJson.options.first.text, 'Flutter');
      expect(fromJson.options.first.votes, 15);
      expect(fromJson.options.first.voted, isTrue);
      expect(fromJson.options.last.votes, 5);
      expect(fromJson.options.last.voted, isFalse);
    });

    test('Message parsing with interactive components', () {
      final json = {
        'id': 'msg-101',
        'channelId': 'ch-1',
        'userId': 1,
        'content': 'Interactive message prompt',
        'createdAt': 1700000000000,
        'components': [
          {
            'type': 'button',
            'id': 'action-btn',
            'label': 'Acknowledge',
            'pluginId': 'alerts',
            'actionName': 'ack'
          },
          {
            'type': 'poll',
            'id': 'poll-2',
            'question': 'Ready for release?',
            'options': [
              {'id': 'yes', 'text': 'Yes', 'votes': 8},
              {'id': 'no', 'text': 'No', 'votes': 2}
            ]
          }
        ]
      };

      final message = Message.fromJson(json);
      expect(message.components.length, 2);
      expect(message.components[0].isButton, isTrue);
      expect(message.components[0].button?.label, 'Acknowledge');
      expect(message.components[1].isPoll, isTrue);
      expect(message.components[1].poll?.question, 'Ready for release?');
      expect(message.components[1].poll?.options.length, 2);
    });
  });

  group('ChatCanvas Interactive Components & Slash Commands Widget Tests', () {
    testWidgets('renders interactive buttons and polls inside message stream',
        (tester) async {
      const channel = Channel(
        id: '1',
        name: 'general',
        isVoice: false,
      );

      final message = Message(
        id: 'msg-1',
        channelId: '1',
        author: const User(id: '1', username: 'admin', displayName: 'Admin'),
        content: 'Check out the new features below:',
        timestamp: DateTime.now(),
        components: const [
          InteractiveComponent.button(
            PluginButton(
              id: 'btn-test',
              label: 'Launch Simulator',
              pluginId: 'sim',
              actionName: 'launch',
            ),
          ),
          InteractiveComponent.poll(
            PluginPoll(
              id: 'poll-test',
              question: 'Deploy now?',
              options: [
                PluginPollOption(id: 'yes', text: 'Yes, deploy', votes: 10),
                PluginPollOption(id: 'no', text: 'Wait', votes: 2),
              ],
            ),
          ),
        ],
      );

      final container = ProviderContainer(
        overrides: [
          selectedChannelProvider.overrideWith((ref) => channel),
          messageProvider.overrideWith((ref) {
            final notifier = MessageNotifier();
            notifier.setMessages('1', [message]);
            return notifier;
          }),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: ChatCanvas(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Verify button rendered
      expect(find.text('Launch Simulator'), findsOneWidget);

      // Verify poll rendered
      expect(find.text('Deploy now?'), findsOneWidget);
      expect(find.text('Yes, deploy'), findsOneWidget);
      expect(find.text('Wait'), findsOneWidget);
    });

    testWidgets('typing slash / shows slash command autocomplete popover',
        (tester) async {
      const channel = Channel(
        id: '1',
        name: 'general',
        isVoice: false,
      );

      final container = ProviderContainer(
        overrides: [
          selectedChannelProvider.overrideWith((ref) => channel),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: ChatCanvas(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Enter slash '/' into input field
      final inputFinder = find.byKey(const Key('chat_input_field'));
      expect(inputFinder, findsOneWidget);

      await tester.enterText(inputFinder, '/');
      await tester.pumpAndSettle();

      // Autocomplete list should display built-in/native slash commands
      expect(find.byKey(const Key('slash_command_popover')), findsOneWidget);
      expect(find.text('/poll'), findsOneWidget);
      expect(find.text('/remind'), findsOneWidget);
      expect(find.text('/soundboard'), findsOneWidget);

      // Tap on /poll command from autocomplete list
      await tester.tap(find.text('/poll'));
      await tester.pumpAndSettle();

      // The input field should now have '/poll '
      final inputWidget = tester.widget<TextField>(inputFinder);
      expect(inputWidget.controller?.text, startsWith('/poll'));
    });

    testWidgets('typing slash and characters reactively filters command list',
        (tester) async {
      const channel = Channel(
        id: '1',
        name: 'general',
        isVoice: false,
      );

      final container = ProviderContainer(
        overrides: [
          selectedChannelProvider.overrideWith((ref) => channel),
        ],
      );

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const MaterialApp(
            home: Scaffold(
              body: ChatCanvas(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final inputFinder = find.byKey(const Key('chat_input_field'));
      await tester.enterText(inputFinder, '/');
      await tester.pumpAndSettle();

      expect(find.text('/poll'), findsOneWidget);
      expect(find.text('/remind'), findsOneWidget);
      expect(find.text('/soundboard'), findsOneWidget);

      // Type 'rem' to filter reactively
      await tester.enterText(inputFinder, '/rem');
      await tester.pumpAndSettle();

      expect(find.text('/remind'), findsOneWidget);
      expect(find.text('/poll'), findsNothing);
      expect(find.text('/soundboard'), findsNothing);
    });
  });
}
