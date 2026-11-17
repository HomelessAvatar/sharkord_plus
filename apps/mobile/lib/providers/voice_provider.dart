import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/models/voice_state.dart';

/// Notifier managing real-time voice channel connection and audio controls.
class VoiceNotifier extends StateNotifier<VoiceState> {
  VoiceNotifier() : super(const VoiceState());

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
      pingMs: 24,
      participants: participants,
      activeSpeakerIds: const [],
    );
  }

  /// Disconnects from active voice channel.
  void leaveVoice() {
    state = const VoiceState();
  }

  /// Toggles microphone mute state.
  void toggleMute([bool? muted]) {
    final newMuted = muted ?? !state.isMuted;
    state = state.copyWith(isMuted: newMuted);
  }

  /// Toggles deafen state. Deafen also mutes microphone automatically.
  void toggleDeafen([bool? deafened]) {
    final newDeafened = deafened ?? !state.isDeafened;
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
  return VoiceNotifier();
});

/// Convenience provider checking if voice is connected.
final isVoiceConnectedProvider = Provider<bool>((ref) {
  return ref.watch(voiceProvider).isConnected;
});

/// Convenience provider for active voice participants.
final voiceParticipantsProvider = Provider<List<User>>((ref) {
  return ref.watch(voiceProvider).participants;
});
