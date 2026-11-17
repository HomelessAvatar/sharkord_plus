import 'package:flutter/foundation.dart';

/// User representation in the mobile application.
@immutable
class User {
  final String id;
  final String username;
  final String displayName;
  final String? avatarUrl;
  final bool isOnline;

  const User({
    required this.id,
    required this.username,
    required this.displayName,
    this.avatarUrl,
    this.isOnline = false,
  });

  /// Numeric identifier if parseable.
  int get numericId => int.tryParse(id) ?? 0;

  User copyWith({
    String? id,
    String? username,
    String? displayName,
    String? avatarUrl,
    bool? isOnline,
  }) {
    return User(
      id: id ?? this.id,
      username: username ?? this.username,
      displayName: displayName ?? this.displayName,
      avatarUrl: avatarUrl ?? this.avatarUrl,
      isOnline: isOnline ?? this.isOnline,
    );
  }

  factory User.fromJson(Map<String, dynamic> json) {
    return User(
      id: json['id']?.toString() ?? '',
      username: json['username'] as String? ?? json['name'] as String? ?? '',
      displayName: json['displayName'] as String? ??
          json['identity'] as String? ??
          json['username'] as String? ??
          json['name'] as String? ??
          '',
      avatarUrl: json['avatarUrl'] as String?,
      isOnline: json['isOnline'] as bool? ??
          (json['status'] != null && json['status'] != 'offline'),
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'username': username,
        'displayName': displayName,
        'avatarUrl': avatarUrl,
        'isOnline': isOnline,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is User &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          username == other.username &&
          displayName == other.displayName &&
          avatarUrl == other.avatarUrl &&
          isOnline == other.isOnline;

  @override
  int get hashCode =>
      Object.hash(id, username, displayName, avatarUrl, isOnline);

  @override
  String toString() =>
      'User(id: $id, username: $username, displayName: $displayName, isOnline: $isOnline)';
}
