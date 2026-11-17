import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/models/message.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';

/// State of chat messages indexed by channel ID.
@immutable
class MessageState {
  final Map<String, List<Message>> messagesByChannel;
  final bool isLoading;
  final String? error;

  const MessageState({
    this.messagesByChannel = const {},
    this.isLoading = false,
    this.error,
  });

  /// Retrieves messages for a specific channel.
  List<Message> getMessages(String channelId) =>
      messagesByChannel[channelId] ?? const [];

  MessageState copyWith({
    Map<String, List<Message>>? messagesByChannel,
    bool? isLoading,
    String? error,
    bool clearError = false,
  }) {
    return MessageState(
      messagesByChannel: messagesByChannel ?? this.messagesByChannel,
      isLoading: isLoading ?? this.isLoading,
      error: clearError ? null : (error ?? this.error),
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is MessageState &&
          runtimeType == other.runtimeType &&
          mapEquals(messagesByChannel, other.messagesByChannel) &&
          isLoading == other.isLoading &&
          error == other.error;

  @override
  int get hashCode => Object.hash(
        Object.hashAll(messagesByChannel.entries),
        isLoading,
        error,
      );
}

/// Notifier managing message history, optimistic sends, and incoming events.
class MessageNotifier extends StateNotifier<MessageState> {
  final Ref? _ref;

  MessageNotifier([this._ref]) : super(const MessageState());

  /// Sets message list for a specific channel.
  void setMessages(String channelId, List<Message> messages) {
    final updated = Map<String, List<Message>>.from(state.messagesByChannel);
    updated[channelId] = List.from(messages);
    state = state.copyWith(messagesByChannel: updated);
  }

  /// Appends or updates a message in its channel.
  void addMessage(Message message) {
    final updated = Map<String, List<Message>>.from(state.messagesByChannel);
    final currentList = List<Message>.from(updated[message.channelId] ?? const []);

    final existingIndex = currentList.indexWhere((m) => m.id == message.id);
    if (existingIndex >= 0) {
      currentList[existingIndex] = message;
    } else {
      currentList.add(message);
    }

    updated[message.channelId] = currentList;
    state = state.copyWith(messagesByChannel: updated);
  }

  /// Sends a message with optimistic UI rendering and live socket propagation.
  Future<Message> sendMessage({
    required String channelId,
    required String content,
    required User author,
  }) async {
    final cleanContent = content.trim();
    if (cleanContent.isEmpty) {
      throw ArgumentError('Message content cannot be empty');
    }

    final tempId = 'temp_${DateTime.now().millisecondsSinceEpoch}';
    final optimisticMessage = Message(
      id: tempId,
      channelId: channelId,
      author: author,
      content: cleanContent,
      timestamp: DateTime.now(),
      attachments: const [],
      reactions: const [],
    );

    // Optimistically add to channel message list immediately
    addMessage(optimisticMessage);

    try {
      final session = _ref?.read(socketSessionProvider);
      final numericChannelId = int.tryParse(channelId);

      Message confirmedMessage;
      if (session != null && numericChannelId != null) {
        // Send through real WebSocket session
        confirmedMessage = await session.sendMessage(numericChannelId, cleanContent);
      } else {
        // Fallback simulation for offline / testing
        await Future<void>.delayed(const Duration(milliseconds: 15));
        final confirmedId = 'msg_${DateTime.now().millisecondsSinceEpoch}';
        confirmedMessage = optimisticMessage.copyWith(id: confirmedId);
      }

      // Replace optimistic message with confirmed server message
      final updated = Map<String, List<Message>>.from(state.messagesByChannel);
      final currentList = List<Message>.from(updated[channelId] ?? const []);
      final tempIndex = currentList.indexWhere((m) => m.id == tempId);

      if (tempIndex >= 0) {
        currentList[tempIndex] = confirmedMessage;
      } else {
        currentList.add(confirmedMessage);
      }

      updated[channelId] = currentList;
      state = state.copyWith(messagesByChannel: updated);

      return confirmedMessage;
    } catch (e) {
      state = state.copyWith(error: e.toString());
      rethrow;
    }
  }

  /// Receives an incoming message (e.g. from websocket subscription).
  void receiveMessage(Message message) {
    addMessage(message);
  }

  /// Deletes a message from a channel.
  void deleteMessage(String channelId, String messageId) {
    final updated = Map<String, List<Message>>.from(state.messagesByChannel);
    final currentList = List<Message>.from(updated[channelId] ?? const []);
    currentList.removeWhere((m) => m.id == messageId);
    updated[channelId] = currentList;
    state = state.copyWith(messagesByChannel: updated);
  }

  /// Clears all messages in a channel.
  void clearChannelMessages(String channelId) {
    final updated = Map<String, List<Message>>.from(state.messagesByChannel);
    updated.remove(channelId);
    state = state.copyWith(messagesByChannel: updated);
  }
}

/// Provider for message state.
final messageProvider =
    StateNotifierProvider<MessageNotifier, MessageState>((ref) {
  return MessageNotifier(ref);
});

/// Family provider for observing messages of a specific channel.
final channelMessagesProvider =
    Provider.family<List<Message>, String>((ref, channelId) {
  return ref.watch(messageProvider).getMessages(channelId);
});
