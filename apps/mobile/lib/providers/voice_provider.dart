import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/voice/webrtc_voice_engine.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/models/voice_state.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';

/// Provider for WebRTC voice engine instance.
final voiceEngineProvider = Provider<WebRTCVoiceEngine>((ref) {
  final session = ref.watch(socketSessionProvider);
  return WebRTCVoiceEngine(socketSession: session);
});

/// Notifier managing real-time voice channel connection and audio controls.
class VoiceNotifier extends StateNotifier<VoiceState> {
  final WebRTCVoiceEngine _engine;
  final Ref? _ref;

  VoiceNotifier({WebRTCVoiceEngine? engine, Ref? ref})
      : _engine = engine ?? WebRTCVoiceEngine(),
        _ref = ref,
        super(const VoiceState()) {
    _bindEngine();
  }

  /// Active WebRTC voice engine instance.
  WebRTCVoiceEngine get engine => _engine;

  void _bindEngine() {
    _engine.onPingChanged = (ping) {
      updatePing(ping);
    };
    _engine.onSpeakingChanged = (userId, isSpeaking) {
      setSpeaking(userId, isSpeaking);
    };
  }

  /// Connects to a voice channel.
  void joinVoice({
    required String channelId,
    required String channelName,
    User? currentUser,
  }) {
    final participants = <User>[];
    if (currentUser != null) {
      participants.add(currentUser);
    }

    state = state.copyWith(
      channelId: channelId,
      channelName: channelName,
      isConnected: true,
      pingMs: 0,
      participants: participants,
      activeSpeakerIds: const [],
    );

    final numericId = int.tryParse(channelId);
    if (numericId != null) {
      final session = _ref?.read(socketSessionProvider);
      _engine.join(
        channelId: numericId,
        userId: currentUser?.id ?? '',
        socketSession: session,
      );
    }
  }

  /// Disconnects from active voice channel.
  void leaveVoice() {
    _engine.leave();
    state = const VoiceState();
  }

  /// Toggles microphone mute state.
  void toggleMute([bool? muted]) {
    final newMuted = muted ?? !state.isMuted;
    _engine.setMuted(newMuted);
    state = state.copyWith(isMuted: newMuted);
  }

  /// Toggles deafen state. Deafen also mutes microphone automatically.
  void toggleDeafen([bool? deafened]) {
    final newDeafened = deafened ?? !state.isDeafened;
    _engine.setDeafened(newDeafened);
    state = state.copyWith(
      isDeafened: newDeafened,
      isMuted: newDeafened ? true : state.isMuted,
    );
  }

  /// Updates active speaker state for a user.
  void setSpeaking(String userId, bool isSpeaking) {
    final currentSpeakers = Set<String>.from(state.activeSpeakerIds);
    if (isSpeaking) {
      currentSpeakers.add(userId);
    } else {
      currentSpeakers.remove(userId);
    }
    state = state.copyWith(activeSpeakerIds: currentSpeakers.toList());
  }

  /// Updates connection latency in milliseconds.
  void updatePing(int pingMs) {
    state = state.copyWith(pingMs: pingMs);
  }

  /// Updates entire participants list.
  void updateParticipants(List<User> participants) {
    state = state.copyWith(participants: participants);
  }

  /// Adds a participant to the channel.
  void addParticipant(User user) {
    if (state.participants.any((p) => p.id == user.id)) return;
    state = state.copyWith(participants: [...state.participants, user]);
  }

  /// Removes a participant from the channel.
  void removeParticipant(String userId) {
    final updated = state.participants.where((p) => p.id != userId).toList();
    final updatedSpeakers =
        state.activeSpeakerIds.where((id) => id != userId).toList();
    state = state.copyWith(
      participants: updated,
      activeSpeakerIds: updatedSpeakers,
    );
  }
}

/// Provider for voice connection state.
final voiceProvider = StateNotifierProvider<VoiceNotifier, VoiceState>((ref) {
  final engine = ref.watch(voiceEngineProvider);
  return VoiceNotifier(engine: engine, ref: ref);
});

/// Convenience provider checking if voice is connected.
final isVoiceConnectedProvider = Provider<bool>((ref) {
  return ref.watch(voiceProvider).isConnected;
});

/// Convenience provider for active voice participants.
final voiceParticipantsProvider = Provider<List<User>>((ref) {
  return ref.watch(voiceProvider).participants;
});
