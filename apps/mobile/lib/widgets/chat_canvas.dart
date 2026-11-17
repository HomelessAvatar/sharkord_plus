import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/models/message.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/message_provider.dart';

/// Central chat canvas displaying scrollable message feed and input composer.
class ChatCanvas extends ConsumerStatefulWidget {
  const ChatCanvas({super.key});

  @override
  ConsumerState<ChatCanvas> createState() => _ChatCanvasState();
}

class _ChatCanvasState extends ConsumerState<ChatCanvas> {
  final _messageController = TextEditingController();
  final _scrollController = ScrollController();

  @override
  void dispose() {
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  Future<void> _handleSendMessage(String channelId) async {
    final text = _messageController.text.trim();
    if (text.isEmpty) return;

    final currentUser = ref.read(currentUserProvider) ??
        const User(
          id: '1',
          username: 'user',
          displayName: 'User',
          isOnline: true,
        );

    _messageController.clear();

    try {
      await ref.read(messageProvider.notifier).sendMessage(
            channelId: channelId,
            content: text,
            author: currentUser,
          );

      // Scroll to bottom after new message
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent + 80,
          duration: const Duration(milliseconds: 250),
          curve: Curves.easeOut,
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to send message: $e'),
            backgroundColor: SharkordColors.danger,
          ),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final selectedChannel = ref.watch(selectedChannelProvider);
    final channelId = selectedChannel?.id ?? '1';
    final channelName = selectedChannel?.name ?? 'general';

    final messages = ref.watch(messageProvider).getMessages(channelId);

    return Container(
      color: SharkordColors.background,
      child: Column(
        children: [
          // Message List Feed
          Expanded(
            child: messages.isEmpty
                ? _buildEmptyState(channelName)
                : ListView.builder(
                    controller: _scrollController,
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 16,
                    ),
                    itemCount: messages.length,
                    itemBuilder: (context, index) {
                      final message = messages[index];
                      return _buildMessageTile(message);
                    },
                  ),
          ),

          // Message Composer Bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: const BoxDecoration(
              color: SharkordColors.surface,
              border: Border(
                top: BorderSide(color: SharkordColors.border),
              ),
            ),
            child: SafeArea(
              top: false,
              child: Row(
                children: [
                  IconButton(
                    icon: const Icon(
                      Icons.add_circle_outline_rounded,
                      color: SharkordColors.textMuted,
                      size: 24,
                    ),
                    onPressed: () {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(
                          content: Text('Attachments are supported via API'),
                          duration: Duration(seconds: 1),
                        ),
                      );
                    },
                  ),
                  Expanded(
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 14),
                      decoration: BoxDecoration(
                        color: SharkordColors.card,
                        borderRadius: BorderRadius.circular(20),
                        border: Border.all(color: SharkordColors.border),
                      ),
                      child: TextField(
                        key: const Key('chat_input_field'),
                        controller: _messageController,
                        style: const TextStyle(
                          color: SharkordColors.textPrimary,
                          fontSize: 14,
                        ),
                        maxLines: null,
                        keyboardType: TextInputType.multiline,
                        textInputAction: TextInputAction.send,
                        onSubmitted: (_) => _handleSendMessage(channelId),
                        decoration: InputDecoration(
                          hintText: 'Message #$channelName',
                          hintStyle: const TextStyle(
                            color: SharkordColors.textMuted,
                            fontSize: 14,
                          ),
                          border: InputBorder.none,
                          isDense: true,
                          contentPadding: const EdgeInsets.symmetric(
                            vertical: 10,
                          ),
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton(
                    key: const Key('chat_send_button'),
                    icon: const Icon(
                      Icons.send_rounded,
                      color: SharkordColors.primary,
                      size: 22,
                    ),
                    onPressed: () => _handleSendMessage(channelId),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState(String channelName) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 64,
              height: 64,
              decoration: BoxDecoration(
                color: SharkordColors.surface,
                shape: BoxShape.circle,
                border: Border.all(color: SharkordColors.border),
              ),
              child: const Center(
                child: Icon(
                  Icons.tag_rounded,
                  color: SharkordColors.primary,
                  size: 32,
                ),
              ),
            ),
            const SizedBox(height: 16),
            Text(
              'Welcome to #$channelName!',
              style: const TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.bold,
                color: SharkordColors.textPrimary,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'This is the start of the #$channelName channel.',
              textAlign: TextAlign.center,
              style: const TextStyle(
                fontSize: 14,
                color: SharkordColors.textMuted,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMessageTile(Message message) {
    final author = message.author;
    final initial = author.displayName.isNotEmpty
        ? author.displayName[0].toUpperCase()
        : 'U';

    final timeString =
        '${message.timestamp.hour.toString().padLeft(2, '0')}:${message.timestamp.minute.toString().padLeft(2, '0')}';

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          CircleAvatar(
            radius: 18,
            backgroundColor: SharkordColors.card,
            child: Text(
              initial,
              style: const TextStyle(
                color: SharkordColors.textPrimary,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  crossAxisAlignment: CrossAxisAlignment.baseline,
                  textBaseline: TextBaseline.alphabetic,
                  children: [
                    Text(
                      author.displayName,
                      style: const TextStyle(
                        color: SharkordColors.textPrimary,
                        fontWeight: FontWeight.bold,
                        fontSize: 14,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Text(
                      timeString,
                      style: const TextStyle(
                        color: SharkordColors.textMuted,
                        fontSize: 11,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  message.content,
                  style: const TextStyle(
                    color: SharkordColors.textPrimary,
                    fontSize: 14,
                    height: 1.35,
                  ),
                ),
                if (message.attachments.isNotEmpty) ...[
                  const SizedBox(height: 6),
                  Wrap(
                    spacing: 6,
                    runSpacing: 4,
                    children: message.attachments.map((attachment) {
                      return Chip(
                        label: Text(
                          attachment,
                          style: const TextStyle(
                            fontSize: 11,
                            color: SharkordColors.textMuted,
                          ),
                        ),
                        backgroundColor: SharkordColors.card,
                        side: const BorderSide(color: SharkordColors.border),
                      );
                    }).toList(),
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
