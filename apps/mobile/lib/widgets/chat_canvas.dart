import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/models/message.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/message_provider.dart';

/// Central chat canvas displaying scrollable message feed, native interactive
/// components (buttons, polls), and slash command autocomplete.
class ChatCanvas extends ConsumerStatefulWidget {
  const ChatCanvas({super.key});

  @override
  ConsumerState<ChatCanvas> createState() => _ChatCanvasState();
}

class _ChatCanvasState extends ConsumerState<ChatCanvas> {
  final _messageController = TextEditingController();
  final _scrollController = ScrollController();
  bool _showSlashAutocomplete = false;

  static const List<PluginCommand> _defaultSlashCommands = [
    PluginCommand(
      pluginId: 'poll',
      name: 'poll',
      description: 'Create an interactive poll',
    ),
    PluginCommand(
      pluginId: 'remind',
      name: 'remind',
      description: 'Set a reminder notification',
    ),
    PluginCommand(
      pluginId: 'soundboard',
      name: 'soundboard',
      description: 'Play a sound clip into voice',
    ),
  ];

  @override
  void initState() {
    super.initState();
    _messageController.addListener(_onTextChanged);
  }

  void _onTextChanged() {
    final text = _messageController.text;
    final isSlash = text.startsWith('/');
    if (isSlash != _showSlashAutocomplete) {
      setState(() {
        _showSlashAutocomplete = isSlash;
      });
    }
  }

  @override
  void dispose() {
    _messageController.removeListener(_onTextChanged);
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  List<PluginCommand> _getMatchingCommands() {
    final query = _messageController.text.trim();
    if (!query.startsWith('/')) return const [];
    final search = query.substring(1).toLowerCase();
    if (search.isEmpty) return _defaultSlashCommands;
    return _defaultSlashCommands
        .where((cmd) => cmd.name.toLowerCase().startsWith(search))
        .toList();
  }

  Future<void> _handlePluginAction({
    required String pluginId,
    required String actionName,
    dynamic payload,
    String? channelId,
  }) async {
    try {
      final session = ref.read(socketSessionProvider);
      final numericChannelId = int.tryParse(channelId ?? '');
      if (session != null) {
        await session.callPluginAction(
          pluginId: pluginId,
          actionName: actionName,
          payload: payload,
          channelId: numericChannelId,
        );
      }
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Action "$actionName" triggered'),
            duration: const Duration(seconds: 1),
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Plugin action failed: $e'),
            backgroundColor: SharkordColors.danger,
          ),
        );
      }
    }
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
    setState(() {
      _showSlashAutocomplete = false;
    });

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

          // Slash Command Autocomplete Popover
          if (_showSlashAutocomplete) _buildSlashAutocompletePopover(),

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

  Widget _buildSlashAutocompletePopover() {
    final matching = _getMatchingCommands();
    if (matching.isEmpty) return const SizedBox.shrink();

    return Container(
      key: const Key('slash_command_popover'),
      constraints: const BoxConstraints(maxHeight: 180),
      margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: SharkordColors.border),
        boxShadow: const [
          BoxShadow(
            color: Colors.black45,
            blurRadius: 8,
            offset: Offset(0, -2),
          ),
        ],
      ),
      child: Material(
        color: SharkordColors.card,
        borderRadius: BorderRadius.circular(12),
        clipBehavior: Clip.antiAlias,
        child: ListView.separated(
          shrinkWrap: true,
          padding: const EdgeInsets.symmetric(vertical: 4),
          itemCount: matching.length,
          separatorBuilder: (_, __) =>
              const Divider(height: 1, color: SharkordColors.border),
          itemBuilder: (context, index) {
            final cmd = matching[index];
            return ListTile(
              dense: true,
              visualDensity: VisualDensity.compact,
              title: Text(
                '/${cmd.name}',
                style: const TextStyle(
                  color: SharkordColors.primary,
                  fontWeight: FontWeight.bold,
                  fontSize: 13,
                ),
              ),
              subtitle: Text(
                cmd.description,
                style: const TextStyle(
                  color: SharkordColors.textMuted,
                  fontSize: 11,
                ),
              ),
              trailing: Text(
                cmd.pluginId,
                style: const TextStyle(
                  color: SharkordColors.textMuted,
                  fontSize: 10,
                ),
              ),
              onTap: () {
                setState(() {
                  _messageController.text = '/${cmd.name} ';
                  _messageController.selection = TextSelection.fromPosition(
                    TextPosition(offset: _messageController.text.length),
                  );
                  _showSlashAutocomplete = false;
                });
              },
            );
          },
        ),
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

  Widget _buildInteractiveButton(PluginButton button, String channelId) {
    final isDanger = button.style == 'danger';
    final isPrimary = button.style == 'primary' || button.style == null;

    Color bgColor = SharkordColors.card;
    Color textColor = SharkordColors.textPrimary;
    BorderSide borderSide = const BorderSide(color: SharkordColors.border);

    if (isDanger) {
      bgColor = SharkordColors.danger.withOpacity(0.15);
      textColor = SharkordColors.danger;
      borderSide = BorderSide(color: SharkordColors.danger.withOpacity(0.5));
    } else if (isPrimary) {
      bgColor = SharkordColors.primary;
      textColor = Colors.white;
      borderSide = BorderSide.none;
    }

    return Padding(
      padding: const EdgeInsets.only(top: 4, bottom: 4),
      child: OutlinedButton(
        style: OutlinedButton.styleFrom(
          backgroundColor: bgColor,
          foregroundColor: textColor,
          side: borderSide,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
        ),
        onPressed: button.disabled
            ? null
            : () {
                if (button.pluginId != null && button.actionName != null) {
                  _handlePluginAction(
                    pluginId: button.pluginId!,
                    actionName: button.actionName!,
                    payload: button.payload,
                    channelId: channelId,
                  );
                }
              },
        child: Text(
          button.label,
          style: TextStyle(
            fontSize: 13,
            fontWeight: FontWeight.w600,
            color: textColor,
          ),
        ),
      ),
    );
  }

  Widget _buildInteractivePoll(PluginPoll poll, String channelId) {
    final totalVotes = poll.totalVotes > 0 ? poll.totalVotes : 1;

    return Container(
      margin: const EdgeInsets.only(top: 6, bottom: 6),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: SharkordColors.surface,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: SharkordColors.border),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(
                Icons.poll_rounded,
                color: SharkordColors.primary,
                size: 18,
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  poll.question,
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.bold,
                    color: SharkordColors.textPrimary,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          ...poll.options.map((option) {
            final pct = (option.votes / totalVotes).clamp(0.0, 1.0);
            final isVoted = option.voted;

            return Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: InkWell(
                borderRadius: BorderRadius.circular(8),
                onTap: poll.closed
                    ? null
                    : () {
                        if (poll.pluginId != null && poll.actionName != null) {
                          _handlePluginAction(
                            pluginId: poll.pluginId!,
                            actionName: poll.actionName!,
                            payload: {'pollId': poll.id, 'optionId': option.id},
                            channelId: channelId,
                          );
                        }
                      },
                child: Container(
                  decoration: BoxDecoration(
                    color: isVoted
                        ? SharkordColors.primary.withOpacity(0.12)
                        : SharkordColors.card,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(
                      color: isVoted
                          ? SharkordColors.primary
                          : SharkordColors.border,
                    ),
                  ),
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(8),
                    child: Stack(
                      children: [
                        Positioned.fill(
                          child: FractionallySizedBox(
                            alignment: Alignment.centerLeft,
                            widthFactor: pct,
                            child: Container(
                              color: isVoted
                                  ? SharkordColors.primary.withOpacity(0.25)
                                  : SharkordColors.surface.withOpacity(0.5),
                            ),
                          ),
                        ),
                        Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 12,
                            vertical: 10,
                          ),
                          child: Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Text(
                                option.text,
                                style: TextStyle(
                                  fontSize: 13,
                                  fontWeight: isVoted
                                      ? FontWeight.bold
                                      : FontWeight.normal,
                                  color: SharkordColors.textPrimary,
                                ),
                              ),
                              Text(
                                '${option.votes} (${(pct * 100).toStringAsFixed(0)}%)',
                                style: const TextStyle(
                                  fontSize: 12,
                                  color: SharkordColors.textMuted,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            );
          }),
        ],
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
                if (message.components.isNotEmpty) ...[
                  const SizedBox(height: 8),
                  ...message.components.map((component) {
                    if (component.isButton && component.button != null) {
                      return _buildInteractiveButton(
                        component.button!,
                        message.channelId,
                      );
                    } else if (component.isPoll && component.poll != null) {
                      return _buildInteractivePoll(
                        component.poll!,
                        message.channelId,
                      );
                    }
                    return const SizedBox.shrink();
                  }),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
