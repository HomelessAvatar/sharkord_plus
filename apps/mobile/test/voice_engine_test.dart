import 'package:flutter_test/flutter_test.dart';
import 'package:sharkord_mobile/core/voice/webrtc_voice_engine.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';

void main() {
  group('Voice Engine & Dynamic Ping Tests', () {
    test('VoiceNotifier updates dynamic ping and active speaker states', () {
      final notifier = VoiceNotifier();
      expect(notifier.state.isConnected, false);
      expect(notifier.state.pingMs, 0);

      notifier.joinVoice(
        channelId: 'voice_1',
        channelName: 'General Voice',
        currentUser: const User(id: '1', username: 'alex', displayName: 'Alex'),
      );

      expect(notifier.state.isConnected, true);
      // Real dynamic ping: must start at 0 until RTT is measured, never static 24ms
      expect(notifier.state.pingMs, 0);

      // Dynamic ping update from WebRTC stats
      notifier.updatePing(48);
      expect(notifier.state.pingMs, 48);

      notifier.setSpeaking('1', true);
      expect(notifier.state.activeSpeakerIds, contains('1'));

      notifier.setSpeaking('1', false);
      expect(notifier.state.activeSpeakerIds, isNot(contains('1')));

      notifier.leaveVoice();
      expect(notifier.state.isConnected, false);
      expect(notifier.state.pingMs, 0);
    });

    test('WebRTCVoiceEngine initializes in test mode and handles state transitions', () async {
      final engine = WebRTCVoiceEngine();
      expect(engine.isTestMode, isTrue);
      expect(engine.isConnected, isFalse);

      int lastPing = -1;
      final speakerUpdates = <String, bool>{};

      engine.onPingChanged = (ping) {
        lastPing = ping;
      };
      engine.onSpeakingChanged = (userId, speaking) {
        speakerUpdates[userId] = speaking;
      };

      await engine.join(
        channelId: 101,
        userId: '1',
      );

      expect(engine.isConnected, isTrue);

      // Simulate ping change from stats
      engine.simulatePing(32);
      expect(lastPing, 32);

      // Simulate speaking change
      engine.simulateSpeaking('1', true);
      expect(speakerUpdates['1'], isTrue);

      engine.setMuted(true);
      expect(engine.isMuted, isTrue);

      engine.setDeafened(true);
      expect(engine.isDeafened, isTrue);

      await engine.leave();
      expect(engine.isConnected, isFalse);
    });

    test('VoiceNotifier wires with WebRTCVoiceEngine for live session', () async {
      final engine = WebRTCVoiceEngine();
      final notifier = VoiceNotifier(engine: engine);

      expect(notifier.state.isConnected, isFalse);
      expect(notifier.state.pingMs, 0);

      notifier.joinVoice(
        channelId: '101',
        channelName: 'Lounge',
        currentUser: const User(id: '1', username: 'alex', displayName: 'Alex'),
      );

      expect(notifier.state.isConnected, isTrue);
      expect(notifier.state.pingMs, 0);

      // Dynamic ping emitted by engine updates notifier state
      engine.simulatePing(27);
      expect(notifier.state.pingMs, 27);

      // Dynamic speaking emitted by engine updates notifier state
      engine.simulateSpeaking('1', true);
      expect(notifier.state.activeSpeakerIds, contains('1'));

      engine.simulateSpeaking('1', false);
      expect(notifier.state.activeSpeakerIds, isNot(contains('1')));

      notifier.toggleMute(true);
      expect(engine.isMuted, isTrue);

      notifier.toggleDeafen(true);
      expect(engine.isDeafened, isTrue);

      notifier.leaveVoice();
      expect(notifier.state.isConnected, isFalse);
      expect(notifier.state.pingMs, 0);
      expect(engine.isConnected, isFalse);
    });
  });
}
