import 'package:flutter/foundation.dart';
import 'package:sharkord_mobile/models/user.dart';

/// Message representation in the mobile application.
@immutable
class Message {
  final String id;
  final String channelId;
  final User author;
  final String content;
  final DateTime timestamp;
  final List<String> attachments;
  final List<String> reactions;

  const Message({
    required this.id,
    required this.channelId,
    required this.author,
    required this.content,
    required this.timestamp,
    this.attachments = const [],
    this.reactions = const [],
  });

  /// Numeric identifier if parseable.
  int get numericId => int.tryParse(id) ?? 0;

  /// Numeric channel identifier if parseable.
  int get numericChannelId => int.tryParse(channelId) ?? 0;

  /// Timestamp in milliseconds since Unix epoch.
  int get timestampMillis => timestamp.millisecondsSinceEpoch;

  Message copyWith({
    String? id,
    String? channelId,
    User? author,
    String? content,
    DateTime? timestamp,
    List<String>? attachments,
    List<String>? reactions,
  }) {
    return Message(
      id: id ?? this.id,
      channelId: channelId ?? this.channelId,
      author: author ?? this.author,
      content: content ?? this.content,
      timestamp: timestamp ?? this.timestamp,
      attachments: attachments ?? this.attachments,
      reactions: reactions ?? this.reactions,
    );
  }

  factory Message.fromJson(Map<String, dynamic> json) {
    DateTime parsedTimestamp;
    final rawTs = json['timestamp'] ?? json['createdAt'];
    if (rawTs is int) {
      if (rawTs < 10000000000) {
        parsedTimestamp = DateTime.fromMillisecondsSinceEpoch(rawTs * 1000);
      } else {
        parsedTimestamp = DateTime.fromMillisecondsSinceEpoch(rawTs);
      }
    } else if (rawTs is String) {
      parsedTimestamp = DateTime.tryParse(rawTs) ?? DateTime.now();
    } else {
      parsedTimestamp = DateTime.now();
    }

    final authorData = json['author'];
    final author = authorData is Map<String, dynamic>
        ? User.fromJson(authorData)
        : User(
            id: json['userId']?.toString() ?? '0',
            username: 'User ${json['userId'] ?? 0}',
            displayName: 'User ${json['userId'] ?? 0}',
          );

    return Message(
      id: json['id']?.toString() ?? '',
      channelId: json['channelId']?.toString() ?? '',
      author: author,
      content: json['content'] as String? ?? '',
      timestamp: parsedTimestamp,
      attachments: (json['attachments'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const [],
      reactions: (json['reactions'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const [],
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'channelId': channelId,
        'author': author.toJson(),
        'content': content,
        'timestamp': timestamp.millisecondsSinceEpoch,
        'attachments': attachments,
        'reactions': reactions,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Message &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          channelId == other.channelId &&
          author == other.author &&
          content == other.content &&
          timestamp == other.timestamp &&
          listEquals(attachments, other.attachments) &&
          listEquals(reactions, other.reactions);

  @override
  int get hashCode => Object.hash(
        id,
        channelId,
        author,
        content,
        timestamp,
        Object.hashAll(attachments),
        Object.hashAll(reactions),
      );

  @override
  String toString() =>
      'Message(id: $id, channelId: $channelId, author: ${author.username}, content: $content)';
}
