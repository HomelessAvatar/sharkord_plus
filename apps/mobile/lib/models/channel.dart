import 'package:flutter/foundation.dart';

/// Channel representation in the mobile application.
@immutable
class Channel {
  final String id;
  final String name;
  final String? category;
  final bool isVoice;
  final int participantCount;
  final List<String> activeSpeakerIds;

  const Channel({
    required this.id,
    required this.name,
    this.category,
    this.isVoice = false,
    this.participantCount = 0,
    this.activeSpeakerIds = const [],
  });

  /// Numeric identifier if parseable.
  int get numericId => int.tryParse(id) ?? 0;

  Channel copyWith({
    String? id,
    String? name,
    String? category,
    bool? isVoice,
    int? participantCount,
    List<String>? activeSpeakerIds,
  }) {
    return Channel(
      id: id ?? this.id,
      name: name ?? this.name,
      category: category ?? this.category,
      isVoice: isVoice ?? this.isVoice,
      participantCount: participantCount ?? this.participantCount,
      activeSpeakerIds: activeSpeakerIds ?? this.activeSpeakerIds,
    );
  }

  factory Channel.fromJson(Map<String, dynamic> json) {
    return Channel(
      id: json['id']?.toString() ?? '',
      name: json['name'] as String? ?? '',
      category: json['category'] as String?,
      isVoice: json['isVoice'] as bool? ??
          (json['type'] == 'VOICE' || json['channelType'] == 'VOICE'),
      participantCount: (json['participantCount'] as num?)?.toInt() ?? 0,
      activeSpeakerIds: (json['activeSpeakerIds'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const [],
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'category': category,
        'isVoice': isVoice,
        'participantCount': participantCount,
        'activeSpeakerIds': activeSpeakerIds,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Channel &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          name == other.name &&
          category == other.category &&
          isVoice == other.isVoice &&
          participantCount == other.participantCount &&
          listEquals(activeSpeakerIds, other.activeSpeakerIds);

  @override
  int get hashCode => Object.hash(
        id,
        name,
        category,
        isVoice,
        participantCount,
        Object.hashAll(activeSpeakerIds),
      );

  @override
  String toString() =>
      'Channel(id: $id, name: $name, isVoice: $isVoice, category: $category, participants: $participantCount)';
}
