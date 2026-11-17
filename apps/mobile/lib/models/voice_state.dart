import 'package:flutter/foundation.dart';
import 'package:sharkord_mobile/models/user.dart';

/// Voice state representation in the mobile application.
@immutable
class VoiceState {
  final String? channelId;
  final String? channelName;
  final bool isConnected;
  final bool isMuted;
  final bool isDeafened;
  final int pingMs;
  final List<User> participants;
  final List<String> activeSpeakerIds;

  const VoiceState({
    this.channelId,
    this.channelName,
    this.isConnected = false,
    this.isMuted = false,
    this.isDeafened = false,
    this.pingMs = 0,
    this.participants = const [],
    this.activeSpeakerIds = const [],
  });

  /// True if the specified user is currently speaking.
  bool isSpeaking(String userId) => activeSpeakerIds.contains(userId);

  /// Numeric channel identifier if parseable.
  int? get numericChannelId =>
      channelId != null ? int.tryParse(channelId!) : null;

  VoiceState copyWith({
    String? channelId,
    String? channelName,
    bool? isConnected,
    bool? isMuted,
    bool? isDeafened,
    int? pingMs,
    List<User>? participants,
    List<String>? activeSpeakerIds,
    bool clearChannel = false,
  }) {
    return VoiceState(
      channelId: clearChannel ? null : (channelId ?? this.channelId),
      channelName: clearChannel ? null : (channelName ?? this.channelName),
      isConnected: isConnected ?? this.isConnected,
      isMuted: isMuted ?? this.isMuted,
      isDeafened: isDeafened ?? this.isDeafened,
      pingMs: pingMs ?? this.pingMs,
      participants: participants ?? this.participants,
      activeSpeakerIds: activeSpeakerIds ?? this.activeSpeakerIds,
    );
  }

  factory VoiceState.fromJson(Map<String, dynamic> json) {
    return VoiceState(
      channelId: json['channelId']?.toString(),
      channelName: json['channelName'] as String?,
      isConnected: json['isConnected'] as bool? ?? false,
      isMuted: json['isMuted'] as bool? ?? false,
      isDeafened: json['isDeafened'] as bool? ?? false,
      pingMs: (json['pingMs'] as num?)?.toInt() ?? 0,
      participants: (json['participants'] as List<dynamic>?)
              ?.map((e) => User.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      activeSpeakerIds: (json['activeSpeakerIds'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const [],
    );
  }

  Map<String, dynamic> toJson() => {
        'channelId': channelId,
        'channelName': channelName,
        'isConnected': isConnected,
        'isMuted': isMuted,
        'isDeafened': isDeafened,
        'pingMs': pingMs,
        'participants': participants.map((p) => p.toJson()).toList(),
        'activeSpeakerIds': activeSpeakerIds,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is VoiceState &&
          runtimeType == other.runtimeType &&
          channelId == other.channelId &&
          channelName == other.channelName &&
          isConnected == other.isConnected &&
          isMuted == other.isMuted &&
          isDeafened == other.isDeafened &&
          pingMs == other.pingMs &&
          listEquals(participants, other.participants) &&
          listEquals(activeSpeakerIds, other.activeSpeakerIds);

  @override
  int get hashCode => Object.hash(
        channelId,
        channelName,
        isConnected,
        isMuted,
        isDeafened,
        pingMs,
        Object.hashAll(participants),
        Object.hashAll(activeSpeakerIds),
      );

  @override
  String toString() =>
      'VoiceState(channel: $channelName ($channelId), connected: $isConnected, muted: $isMuted, deafened: $isDeafened, ping: ${pingMs}ms, participants: ${participants.length})';
}
