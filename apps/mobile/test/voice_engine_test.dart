import 'package:flutter_riverpod/flutter_riverpod.dart';
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

      // Simulate remote user speaking
      engine.simulateSpeaking('2', true);
      expect(speakerUpdates['2'], isTrue);

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

    test('VoiceNotifier.setSpeaking guards against duplicate state transitions (no re-render storm)', () {
      final notifier = VoiceNotifier();
      int stateNotificationCount = 0;

      notifier.addListener((_) {
        stateNotificationCount++;
      });

      // Initial state listener trigger
      expect(stateNotificationCount, 1);

      // First change: user_1 starts speaking
      notifier.setSpeaking('user_1', true);
      expect(stateNotificationCount, 2);
      expect(notifier.state.activeSpeakerIds, contains('user_1'));

      // Duplicate call: user_1 is already speaking -> no state change emitted
      notifier.setSpeaking('user_1', true);
      expect(stateNotificationCount, 2);

      // Duplicate false call for non-speaking user_2 -> no state change emitted
      notifier.setSpeaking('user_2', false);
      expect(stateNotificationCount, 2);

      // Transition user_1 to false -> state change emitted
      notifier.setSpeaking('user_1', false);
      expect(stateNotificationCount, 3);
      expect(notifier.state.activeSpeakerIds, isEmpty);
    });

    test('WebRTCVoiceEngine parses DTLS, RTP parameters and synthesizes remote SDP answers', () {
      final engine = WebRTCVoiceEngine();

      const sampleSdp = '''
v=0\r
o=- 412345 2 IN IP4 127.0.0.1\r
s=-\r
t=0 0\r
a=group:BUNDLE 0\r
m=audio 9 UDP/TLS/RTP/SAVPF 111\r
c=IN IP4 0.0.0.0\r
a=mid:0\r
a=fingerprint:sha-256 AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99\r
a=ssrc:98765 cname:mobile_user_audio\r
''';

      // 1. Extract real DTLS parameters
      final dtls = engine.extractDtlsParameters(sampleSdp, role: 'client');
      expect(dtls['role'], 'client');
      expect(dtls['fingerprints'], isNotEmpty);
      final fp = (dtls['fingerprints'] as List).first as Map<String, dynamic>;
      expect(fp['algorithm'], 'sha-256');
      expect(fp['value'], 'AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99');

      // 2. Extract standard Mediasoup RTP parameters
      final rtp = engine.extractRtpParameters(sampleSdp);
      expect(rtp['mid'], '0');
      expect((rtp['codecs'] as List).first['mimeType'], 'audio/opus');
      expect((rtp['encodings'] as List).first['ssrc'], 98765);
      expect((rtp['rtcp'] as Map)['cname'], 'mobile_user_audio');

      // 3. Build remote answer SDP from Mediasoup transport parameters
      final transportParams = {
        'id': 'transport_test_1',
        'iceParameters': {
          'usernameFragment': 'ufrag123',
          'password': 'pwd456password',
          'iceLite': true,
        },
        'iceCandidates': [
          {
            'foundation': '1',
            'priority': 2130706431,
            'ip': '192.168.1.50',
            'protocol': 'udp',
            'port': 40001,
            'type': 'host',
          }
        ],
        'dtlsParameters': {
          'fingerprints': [
            {
              'algorithm': 'sha-256',
              'value': '11:22:33:44:55:66:77:88:99:00:11:22:33:44:55:66:77:88:99:00:11:22:33:44:55:66:77:88:99:00:11:22',
            }
          ]
        }
      };

      final answerSdp = engine.buildRemoteAnswerSdp(
        localSdp: sampleSdp,
        transportParams: transportParams,
      );

      expect(answerSdp, contains('a=ice-ufrag:ufrag123'));
      expect(answerSdp, contains('a=ice-pwd:pwd456password'));
      expect(answerSdp, contains('a=ice-lite'));
      expect(answerSdp, contains('a=candidate:1 1 udp 2130706431 192.168.1.50 40001 typ host'));
      expect(answerSdp, contains('a=fingerprint:sha-256 11:22:33:44:55:66:77:88:99:00:11:22:33:44:55:66:77:88:99:00:11:22:33:44:55:66:77:88:99:00:11:22'));
    });

    test('voiceEngineProvider cleans up on container disposal', () {
      final container = ProviderContainer();
      final engine = container.read(voiceEngineProvider);
      expect(engine, isNotNull);

      // Disposal of container triggers ref.onDispose on voiceEngineProvider
      container.dispose();
      expect(engine.isConnected, isFalse);
    });
  });
}
