import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:sharkord_mobile/models/user.dart';

/// Represents a registered slash command.
@immutable
class PluginCommand {
  final String pluginId;
  final String name;
  final String description;

  const PluginCommand({
    required this.pluginId,
    required this.name,
    this.description = '',
  });

  factory PluginCommand.fromJson(Map<String, dynamic> json) {
    return PluginCommand(
      pluginId: json['pluginId']?.toString() ??
          json['plugin_id']?.toString() ??
          '',
      name: json['name']?.toString() ?? '',
      description: json['description']?.toString() ?? '',
    );
  }

  Map<String, dynamic> toJson() => {
        'pluginId': pluginId,
        'name': name,
        'description': description,
      };

  PluginCommand copyWith({
    String? pluginId,
    String? name,
    String? description,
  }) {
    return PluginCommand(
      pluginId: pluginId ?? this.pluginId,
      name: name ?? this.name,
      description: description ?? this.description,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PluginCommand &&
          runtimeType == other.runtimeType &&
          pluginId == other.pluginId &&
          name == other.name &&
          description == other.description;

  @override
  int get hashCode => Object.hash(pluginId, name, description);
}

/// Represents an interactive button component rendered inside a message.
@immutable
class PluginButton {
  final String id;
  final String label;
  final String? pluginId;
  final String? actionName;
  final String? style;
  final bool disabled;
  final dynamic payload;

  const PluginButton({
    required this.id,
    required this.label,
    this.pluginId,
    this.actionName,
    this.style,
    this.disabled = false,
    this.payload,
  });

  factory PluginButton.fromJson(Map<String, dynamic> json) {
    return PluginButton(
      id: json['id']?.toString() ?? '',
      label: json['label']?.toString() ?? '',
      pluginId: json['pluginId']?.toString() ?? json['plugin_id']?.toString(),
      actionName: json['actionName']?.toString() ??
          json['action_name']?.toString() ??
          json['actionId']?.toString() ??
          json['action_id']?.toString(),
      style: json['style']?.toString(),
      disabled: json['disabled'] as bool? ?? false,
      payload: json['payload'],
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'label': label,
        if (pluginId != null) 'pluginId': pluginId,
        if (actionName != null) 'actionName': actionName,
        if (style != null) 'style': style,
        'disabled': disabled,
        if (payload != null) 'payload': payload,
      };

  PluginButton copyWith({
    String? id,
    String? label,
    String? pluginId,
    String? actionName,
    String? style,
    bool? disabled,
    dynamic payload,
  }) {
    return PluginButton(
      id: id ?? this.id,
      label: label ?? this.label,
      pluginId: pluginId ?? this.pluginId,
      actionName: actionName ?? this.actionName,
      style: style ?? this.style,
      disabled: disabled ?? this.disabled,
      payload: payload ?? this.payload,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PluginButton &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          label == other.label &&
          pluginId == other.pluginId &&
          actionName == other.actionName &&
          style == other.style &&
          disabled == other.disabled;

  @override
  int get hashCode =>
      Object.hash(id, label, pluginId, actionName, style, disabled);
}

/// An option within an interactive poll.
@immutable
class PluginPollOption {
  final String id;
  final String text;
  final int votes;
  final bool voted;

  const PluginPollOption({
    required this.id,
    required this.text,
    this.votes = 0,
    this.voted = false,
  });

  factory PluginPollOption.fromJson(Map<String, dynamic> json) {
    return PluginPollOption(
      id: json['id']?.toString() ?? '',
      text: json['text']?.toString() ?? '',
      votes: (json['votes'] as num?)?.toInt() ?? 0,
      voted: json['voted'] as bool? ?? false,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'text': text,
        'votes': votes,
        'voted': voted,
      };

  PluginPollOption copyWith({
    String? id,
    String? text,
    int? votes,
    bool? voted,
  }) {
    return PluginPollOption(
      id: id ?? this.id,
      text: text ?? this.text,
      votes: votes ?? this.votes,
      voted: voted ?? this.voted,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PluginPollOption &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          text == other.text &&
          votes == other.votes &&
          voted == other.voted;

  @override
  int get hashCode => Object.hash(id, text, votes, voted);
}

/// An interactive poll component.
@immutable
class PluginPoll {
  final String id;
  final String question;
  final List<PluginPollOption> options;
  final String? pluginId;
  final String? actionName;
  final int totalVotes;
  final bool allowsMultiple;
  final bool closed;

  const PluginPoll({
    required this.id,
    required this.question,
    required this.options,
    this.pluginId,
    this.actionName,
    this.totalVotes = 0,
    this.allowsMultiple = false,
    this.closed = false,
  });

  factory PluginPoll.fromJson(Map<String, dynamic> json) {
    final rawOptions = json['options'];
    final options = (rawOptions is List)
        ? rawOptions
            .whereType<Map<String, dynamic>>()
            .map((e) => PluginPollOption.fromJson(e))
            .toList()
        : <PluginPollOption>[];

    final calculatedVotes = (json['totalVotes'] as num?)?.toInt() ??
        (json['total_votes'] as num?)?.toInt() ??
        options.fold<int>(0, (sum, opt) => sum + opt.votes);

    return PluginPoll(
      id: json['id']?.toString() ?? '',
      question: json['question']?.toString() ?? '',
      options: options,
      pluginId: json['pluginId']?.toString() ?? json['plugin_id']?.toString(),
      actionName: json['actionName']?.toString() ??
          json['action_name']?.toString() ??
          json['actionId']?.toString() ??
          json['action_id']?.toString(),
      totalVotes: calculatedVotes,
      allowsMultiple: json['allowsMultiple'] as bool? ??
          json['allows_multiple'] as bool? ??
          false,
      closed: json['closed'] as bool? ?? false,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'question': question,
        'options': options.map((o) => o.toJson()).toList(),
        if (pluginId != null) 'pluginId': pluginId,
        if (actionName != null) 'actionName': actionName,
        'totalVotes': totalVotes,
        'allowsMultiple': allowsMultiple,
        'closed': closed,
      };

  PluginPoll copyWith({
    String? id,
    String? question,
    List<PluginPollOption>? options,
    String? pluginId,
    String? actionName,
    int? totalVotes,
    bool? allowsMultiple,
    bool? closed,
  }) {
    return PluginPoll(
      id: id ?? this.id,
      question: question ?? this.question,
      options: options ?? this.options,
      pluginId: pluginId ?? this.pluginId,
      actionName: actionName ?? this.actionName,
      totalVotes: totalVotes ?? this.totalVotes,
      allowsMultiple: allowsMultiple ?? this.allowsMultiple,
      closed: closed ?? this.closed,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PluginPoll &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          question == other.question &&
          listEquals(options, other.options) &&
          pluginId == other.pluginId &&
          actionName == other.actionName &&
          totalVotes == other.totalVotes &&
          allowsMultiple == other.allowsMultiple &&
          closed == other.closed;

  @override
  int get hashCode => Object.hash(
        id,
        question,
        Object.hashAll(options),
        pluginId,
        actionName,
        totalVotes,
        allowsMultiple,
        closed,
      );
}

/// Generic wrapper for interactive message components.
@immutable
class InteractiveComponent {
  final String type;
  final PluginButton? button;
  final PluginPoll? poll;

  const InteractiveComponent._({
    required this.type,
    this.button,
    this.poll,
  });

  const InteractiveComponent.button(PluginButton btn)
      : this._(type: 'button', button: btn);

  const InteractiveComponent.poll(PluginPoll p) : this._(type: 'poll', poll: p);

  bool get isButton => type == 'button' && button != null;
  bool get isPoll => type == 'poll' && poll != null;

  factory InteractiveComponent.fromJson(Map<String, dynamic> json) {
    final type = (json['type'] as String?)?.toLowerCase() ?? '';
    if (type == 'button' || json.containsKey('label')) {
      return InteractiveComponent.button(PluginButton.fromJson(json));
    } else if (type == 'poll' || json.containsKey('options')) {
      return InteractiveComponent.poll(PluginPoll.fromJson(json));
    }
    return InteractiveComponent._(type: type);
  }

  Map<String, dynamic> toJson() {
    if (isButton) {
      return {
        'type': 'button',
        ...button!.toJson(),
      };
    } else if (isPoll) {
      return {
        'type': 'poll',
        ...poll!.toJson(),
      };
    }
    return {'type': type};
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is InteractiveComponent &&
          runtimeType == other.runtimeType &&
          type == other.type &&
          button == other.button &&
          poll == other.poll;

  @override
  int get hashCode => Object.hash(type, button, poll);
}

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
  final List<InteractiveComponent> components;

  const Message({
    required this.id,
    required this.channelId,
    required this.author,
    required this.content,
    required this.timestamp,
    this.attachments = const [],
    this.reactions = const [],
    this.components = const [],
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
    List<InteractiveComponent>? components,
  }) {
    return Message(
      id: id ?? this.id,
      channelId: channelId ?? this.channelId,
      author: author ?? this.author,
      content: content ?? this.content,
      timestamp: timestamp ?? this.timestamp,
      attachments: attachments ?? this.attachments,
      reactions: reactions ?? this.reactions,
      components: components ?? this.components,
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

    final rawComponents = json['components'];
    final List<InteractiveComponent> parsedComponents = [];
    if (rawComponents is List) {
      for (final item in rawComponents) {
        if (item is Map<String, dynamic>) {
          parsedComponents.add(InteractiveComponent.fromJson(item));
        }
      }
    }
    if (json['buttons'] is List) {
      for (final btn in json['buttons'] as List) {
        if (btn is Map<String, dynamic>) {
          parsedComponents
              .add(InteractiveComponent.button(PluginButton.fromJson(btn)));
        }
      }
    }
    if (json['poll'] is Map<String, dynamic>) {
      parsedComponents.add(InteractiveComponent.poll(
          PluginPoll.fromJson(json['poll'] as Map<String, dynamic>)));
    }

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
      components: parsedComponents,
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
        'components': components.map((c) => c.toJson()).toList(),
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
          listEquals(reactions, other.reactions) &&
          listEquals(components, other.components);

  @override
  int get hashCode => Object.hash(
        id,
        channelId,
        author,
        content,
        timestamp,
        Object.hashAll(attachments),
        Object.hashAll(reactions),
        Object.hashAll(components),
      );

  @override
  String toString() =>
      'Message(id: $id, channelId: $channelId, author: ${author.username}, content: $content, components: ${components.length})';
}
