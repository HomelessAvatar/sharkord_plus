import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/storage/vault_service.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/models/channel.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/message_provider.dart';
import 'package:sharkord_mobile/providers/server_provider.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';

/// Left Drawer featuring Server Rail and categorized Channel Tree.
class ChannelDrawer extends ConsumerWidget {
  const ChannelDrawer({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final channelState = ref.watch(channelProvider);
    final voiceState = ref.watch(voiceProvider);
    final currentUser = ref.watch(currentUserProvider);
    final serverName = ref.watch(serverNameProvider);
    final currentServerId = ref.watch(serverProvider).serverId;
    final savedServers = ref.watch(savedServersProvider);
    final authState = ref.watch(authProvider);

    // Group channels by category
    final textChannels = channelState.textChannels;
    final voiceChannels = channelState.voiceChannels;

    final displayedServers = savedServers.isNotEmpty
        ? savedServers
        : [
            SavedServerSession(
              serverId: currentServerId.isNotEmpty ? currentServerId : 'default',
              serverName: serverName,
              serverUrl: authState.serverUrl ?? 'https://hq.sharkord.com',
              token: authState.token ?? '',
              userId: currentUser?.id ?? '1',
              username: currentUser?.username ?? 'user',
              displayName: currentUser?.displayName ?? 'User',
            )
          ];

    return Drawer(
      backgroundColor: SharkordColors.sidebar,
      child: SafeArea(
        child: Row(
          children: [
            // Left Server Rail
            Container(
              width: 68,
              color: SharkordColors.background,
              child: Column(
                children: [
                  const SizedBox(height: 12),
                  // Server List Rail
                  Expanded(
                    child: ListView.separated(
                      padding: EdgeInsets.zero,
                      itemCount: displayedServers.length,
                      separatorBuilder: (_, __) => const SizedBox(height: 8),
                      itemBuilder: (context, index) {
                        final server = displayedServers[index];
                        final isActive = server.serverId == currentServerId ||
                            (server.serverUrl == authState.serverUrl &&
                                authState.isAuthenticated) ||
                            (displayedServers.length == 1);

                        return _buildServerRailItem(
                          context: context,
                          ref: ref,
                          server: server,
                          isActive: isActive,
                        );
                      },
                    ),
                  ),
                  const Divider(
                    color: SharkordColors.border,
                    indent: 14,
                    endIndent: 14,
                  ),
                  const SizedBox(height: 8),
                  // Add Server Action
                  Material(
                    color: Colors.transparent,
                    shape: const CircleBorder(),
                    child: InkWell(
                      key: const Key('add_server_rail_button'),
                      customBorder: const CircleBorder(),
                      onTap: () => _showAddServerDialog(context, ref),
                      child: Container(
                        width: 44,
                        height: 44,
                        decoration: BoxDecoration(
                          color: SharkordColors.surface,
                          shape: BoxShape.circle,
                          border: Border.all(color: SharkordColors.border),
                        ),
                        child: const Icon(
                          Icons.add_rounded,
                          color: SharkordColors.speaking,
                          size: 24,
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  // Settings / Profile
                  Material(
                    color: Colors.transparent,
                    shape: const CircleBorder(),
                    child: InkWell(
                      key: const Key('settings_rail_button'),
                      customBorder: const CircleBorder(),
                      onTap: () => _showSettingsDialog(context, ref),
                      child: Container(
                        width: 44,
                        height: 44,
                        margin: const EdgeInsets.only(bottom: 12),
                        decoration: BoxDecoration(
                          color: SharkordColors.surface,
                          shape: BoxShape.circle,
                          border: Border.all(color: SharkordColors.border),
                        ),
                        child: const Icon(
                          Icons.settings_rounded,
                          color: SharkordColors.textMuted,
                          size: 22,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),

            // Channel Tree List
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  // Server Header
                  Container(
                    height: 56,
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    decoration: const BoxDecoration(
                      border: Border(
                        bottom: BorderSide(color: SharkordColors.border),
                      ),
                    ),
                    child: Row(
                      children: [
                        Expanded(
                          child: Text(
                            serverName,
                            style: const TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                              color: SharkordColors.textPrimary,
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        const Icon(
                          Icons.keyboard_arrow_down_rounded,
                          color: SharkordColors.textMuted,
                        ),
                      ],
                    ),
                  ),

                  // Channels List
                  Expanded(
                    child: ListView(
                      padding: const EdgeInsets.symmetric(
                        vertical: 12,
                        horizontal: 8,
                      ),
                      children: [
                        if (textChannels.isEmpty && voiceChannels.isEmpty)
                          Padding(
                            padding: const EdgeInsets.symmetric(
                                vertical: 32, horizontal: 16),
                            child: Column(
                              mainAxisAlignment: MainAxisAlignment.center,
                              children: [
                                const Icon(
                                  Icons.tag_rounded,
                                  color: SharkordColors.textMuted,
                                  size: 32,
                                ),
                                const SizedBox(height: 8),
                                Text(
                                  channelState.isLoading
                                      ? 'Loading channels...'
                                      : 'No channels available',
                                  textAlign: TextAlign.center,
                                  style: const TextStyle(
                                    color: SharkordColors.textMuted,
                                    fontSize: 13,
                                  ),
                                ),
                              ],
                            ),
                          ),
                        // Text Channels Category
                        if (textChannels.isNotEmpty) ...[
                          _buildCategoryHeader('TEXT CHANNELS'),
                          ...textChannels.map((channel) {
                            final isSelected =
                                channelState.selectedChannelId == channel.id;
                            return _buildChannelItem(
                              context: context,
                              ref: ref,
                              channel: channel,
                              isSelected: isSelected,
                              isVoice: false,
                            );
                          }),
                          const SizedBox(height: 16),
                        ],

                        // Voice Channels Category
                        if (voiceChannels.isNotEmpty) ...[
                          _buildCategoryHeader('VOICE CHANNELS'),
                          ...voiceChannels.map((channel) {
                            final isVoiceConnected =
                                voiceState.isConnected &&
                                    voiceState.channelId == channel.id;
                            final hasSpeakers =
                                channel.activeSpeakerIds.isNotEmpty ||
                                    (isVoiceConnected &&
                                        voiceState.activeSpeakerIds.isNotEmpty);

                            return _buildChannelItem(
                              context: context,
                              ref: ref,
                              channel: channel,
                              isSelected: isVoiceConnected,
                              isVoice: true,
                              hasSpeakers: hasSpeakers,
                            );
                          }),
                        ],
                      ],
                    ),
                  ),

                  // Bottom Current User Bar
                  Container(
                    padding: const EdgeInsets.all(10),
                    decoration: const BoxDecoration(
                      color: SharkordColors.surface,
                      border: Border(
                        top: BorderSide(color: SharkordColors.border),
                      ),
                    ),
                    child: Row(
                      children: [
                        CircleAvatar(
                          radius: 18,
                          backgroundColor: SharkordColors.card,
                          child: Text(
                            (currentUser?.displayName.isNotEmpty ?? false)
                                ? currentUser!.displayName[0].toUpperCase()
                                : 'U',
                            style: const TextStyle(
                              color: SharkordColors.textPrimary,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              Text(
                                currentUser?.displayName ?? 'Anonymous User',
                                style: const TextStyle(
                                  color: SharkordColors.textPrimary,
                                  fontWeight: FontWeight.w600,
                                  fontSize: 13,
                                ),
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                              Text(
                                '#${currentUser?.id ?? '0000'}',
                                style: const TextStyle(
                                  color: SharkordColors.textMuted,
                                  fontSize: 11,
                                ),
                              ),
                            ],
                          ),
                        ),
                        IconButton(
                          icon: Icon(
                            voiceState.isMuted
                                ? Icons.mic_off_rounded
                                : Icons.mic_rounded,
                            size: 18,
                            color: voiceState.isMuted
                                ? SharkordColors.danger
                                : SharkordColors.textMuted,
                          ),
                          onPressed: () {
                            ref.read(voiceProvider.notifier).toggleMute();
                          },
                        ),
                        IconButton(
                          icon: Icon(
                            voiceState.isDeafened
                                ? Icons.headset_off_rounded
                                : Icons.headset_rounded,
                            size: 18,
                            color: voiceState.isDeafened
                                ? SharkordColors.danger
                                : SharkordColors.textMuted,
                          ),
                          onPressed: () {
                            ref.read(voiceProvider.notifier).toggleDeafen();
                          },
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildServerRailItem({
    required BuildContext context,
    required WidgetRef ref,
    required SavedServerSession server,
    required bool isActive,
  }) {
    return GestureDetector(
      key: Key('server_rail_item_${server.serverId}'),
      onTap: () {
        if (!isActive) {
          ref.read(authProvider.notifier).switchServer(server);
        }
      },
      onLongPress: () {
        _showServerOptionsDialog(context, ref, server);
      },
      child: Row(
        children: [
          // Active indicator pill
          AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            width: 4,
            height: isActive ? 36 : 8,
            decoration: BoxDecoration(
              color: isActive ? SharkordColors.primary : Colors.transparent,
              borderRadius: const BorderRadius.only(
                topRight: Radius.circular(4),
                bottomRight: Radius.circular(4),
              ),
            ),
          ),
          const SizedBox(width: 8),
          // Server Icon
          Container(
            width: 44,
            height: 44,
            decoration: BoxDecoration(
              color: SharkordColors.surface,
              borderRadius: BorderRadius.circular(isActive ? 14 : 22),
              border: Border.all(
                color: isActive
                    ? SharkordColors.primary.withAlpha(180)
                    : SharkordColors.border,
                width: isActive ? 1.5 : 1.0,
              ),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(isActive ? 12 : 20),
              child: (server.serverName.toLowerCase().contains('sharkord') ||
                      server.serverId == '1' ||
                      server.serverId == 'default')
                  ? Image.asset(
                      'assets/logo.png',
                      width: 44,
                      height: 44,
                      fit: BoxFit.cover,
                    )
                  : Center(
                      child: Text(
                        server.serverName.isNotEmpty
                            ? server.serverName[0].toUpperCase()
                            : 'S',
                        style: TextStyle(
                          color: isActive
                              ? SharkordColors.textPrimary
                              : SharkordColors.textMuted,
                          fontWeight: FontWeight.bold,
                          fontSize: 16,
                        ),
                      ),
                    ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCategoryHeader(String title) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      child: Text(
        title,
        style: const TextStyle(
          color: SharkordColors.textMuted,
          fontSize: 11,
          fontWeight: FontWeight.w700,
          letterSpacing: 0.8,
        ),
      ),
    );
  }

  Widget _buildChannelItem({
    required BuildContext context,
    required WidgetRef ref,
    required Channel channel,
    required bool isSelected,
    required bool isVoice,
    bool hasSpeakers = false,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Material(
        color: isSelected ? SharkordColors.card : Colors.transparent,
        borderRadius: BorderRadius.circular(6),
        child: ListTile(
          dense: true,
          contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 0),
          leading: Icon(
            isVoice ? Icons.volume_up_rounded : Icons.tag_rounded,
            size: 20,
            color: isSelected
                ? (isVoice ? SharkordColors.speaking : SharkordColors.textPrimary)
                : SharkordColors.textMuted,
          ),
          title: Text(
            channel.name,
            style: TextStyle(
              color: isSelected
                  ? SharkordColors.textPrimary
                  : SharkordColors.textMuted,
              fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
              fontSize: 14,
            ),
          ),
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (hasSpeakers)
                Container(
                  width: 8,
                  height: 8,
                  margin: const EdgeInsets.only(right: 6),
                  decoration: const BoxDecoration(
                    color: SharkordColors.speaking,
                    shape: BoxShape.circle,
                    boxShadow: [
                      BoxShadow(
                        color: SharkordColors.speaking,
                        blurRadius: 4,
                        spreadRadius: 1,
                      ),
                    ],
                  ),
                ),
              if (isVoice && channel.participantCount > 0)
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: SharkordColors.surface,
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Text(
                    '${channel.participantCount}',
                    style: const TextStyle(
                      color: SharkordColors.textMuted,
                      fontSize: 11,
                    ),
                  ),
                ),
            ],
          ),
          onTap: () {
            if (isVoice) {
              final currentUser = ref.read(currentUserProvider);
              ref.read(voiceProvider.notifier).joinVoice(
                    channelId: channel.id,
                    channelName: channel.name,
                    currentUser: currentUser,
                  );
              ref
                  .read(channelProvider.notifier)
                  .selectVoiceChannel(channel.id);
            } else {
              ref.read(channelProvider.notifier).selectChannel(channel.id);
              final session = ref.read(socketSessionProvider);
              if (session != null) {
                final numericId = channel.numericId;
                session.fetchMessages(numericId).then((msgs) {
                  ref
                      .read(messageProvider.notifier)
                      .setMessages(channel.id, msgs);
                }).catchError((_) {});
              }
              Navigator.of(context).pop();
            }
          },
        ),
      ),
    );
  }

  void _showServerOptionsDialog(
    BuildContext context,
    WidgetRef ref,
    SavedServerSession server,
  ) {
    showModalBottomSheet<void>(
      context: context,
      backgroundColor: SharkordColors.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      builder: (sheetCtx) {
        return SafeArea(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 20),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  server.serverName,
                  style: const TextStyle(
                    color: SharkordColors.textPrimary,
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  server.serverUrl,
                  style: const TextStyle(
                    color: SharkordColors.textMuted,
                    fontSize: 13,
                  ),
                ),
                const SizedBox(height: 16),
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  leading:
                      const Icon(Icons.sync_alt_rounded, color: SharkordColors.primary),
                  title: const Text(
                    'Switch to this server',
                    style: TextStyle(color: SharkordColors.textPrimary),
                  ),
                  onTap: () {
                    Navigator.of(sheetCtx).pop();
                    ref.read(authProvider.notifier).switchServer(server);
                  },
                ),
                ListTile(
                  key: Key('delete_server_${server.serverId}'),
                  contentPadding: EdgeInsets.zero,
                  leading: const Icon(Icons.delete_outline_rounded,
                      color: SharkordColors.danger),
                  title: const Text(
                    'Delete from Saved Servers',
                    style: TextStyle(color: SharkordColors.danger),
                  ),
                  onTap: () async {
                    final navigator = Navigator.of(context);
                    Navigator.of(sheetCtx).pop();
                    final currentServerId = ref.read(serverProvider).serverId;
                    final isCurrent = server.serverId == currentServerId;

                    await ref
                        .read(savedServersProvider.notifier)
                        .remove(server.serverId);
                    final remaining = ref.read(savedServersProvider);

                    if (isCurrent) {
                      if (remaining.isNotEmpty) {
                        await ref
                            .read(authProvider.notifier)
                            .switchServer(remaining.first);
                      } else {
                        if (context.mounted) {
                          navigator.pop(); // close drawer
                        }
                        ref.read(authProvider.notifier).logout();
                      }
                    }
                  },
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  void _showAddServerDialog(BuildContext context, WidgetRef ref) {
    final urlController = TextEditingController();
    final identityController = TextEditingController();
    final passwordController = TextEditingController();
    var isSubmitting = false;
    String? errorMessage;

    showDialog<void>(
      context: context,
      builder: (dialogCtx) {
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              backgroundColor: SharkordColors.surface,
              title: const Text(
                'Add New Server',
                style: TextStyle(color: SharkordColors.textPrimary, fontSize: 18),
              ),
              content: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Text(
                      'Connect to another Sharkord instance and save it to your multi-server vault.',
                      style:
                          TextStyle(color: SharkordColors.textMuted, fontSize: 13),
                    ),
                    const SizedBox(height: 16),
                    TextField(
                      key: const Key('add_server_url_field'),
                      controller: urlController,
                      autofocus: true,
                      style: const TextStyle(
                          color: SharkordColors.textPrimary, fontSize: 14),
                      decoration: InputDecoration(
                        labelText: 'Server URL',
                        labelStyle:
                            const TextStyle(color: SharkordColors.textMuted),
                        hintText: 'https://hq.sharkord.com',
                        hintStyle:
                            const TextStyle(color: SharkordColors.textMuted),
                        filled: true,
                        fillColor: SharkordColors.card,
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(8),
                          borderSide:
                              const BorderSide(color: SharkordColors.border),
                        ),
                      ),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      key: const Key('add_server_identity_field'),
                      controller: identityController,
                      style: const TextStyle(
                          color: SharkordColors.textPrimary, fontSize: 14),
                      decoration: InputDecoration(
                        labelText: 'Username or Email',
                        labelStyle:
                            const TextStyle(color: SharkordColors.textMuted),
                        filled: true,
                        fillColor: SharkordColors.card,
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(8),
                          borderSide:
                              const BorderSide(color: SharkordColors.border),
                        ),
                      ),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      key: const Key('add_server_password_field'),
                      controller: passwordController,
                      obscureText: true,
                      style: const TextStyle(
                          color: SharkordColors.textPrimary, fontSize: 14),
                      decoration: InputDecoration(
                        labelText: 'Password',
                        labelStyle:
                            const TextStyle(color: SharkordColors.textMuted),
                        filled: true,
                        fillColor: SharkordColors.card,
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(8),
                          borderSide:
                              const BorderSide(color: SharkordColors.border),
                        ),
                      ),
                    ),
                    if (errorMessage != null) ...[
                      const SizedBox(height: 12),
                      Text(
                        errorMessage!,
                        style: const TextStyle(
                            color: SharkordColors.danger, fontSize: 13),
                      ),
                    ],
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(dialogCtx).pop(),
                  child: const Text('Cancel',
                      style: TextStyle(color: SharkordColors.textMuted)),
                ),
                ElevatedButton(
                  key: const Key('add_server_submit_button'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: SharkordColors.primary,
                    foregroundColor: Colors.white,
                  ),
                  onPressed: isSubmitting
                      ? null
                      : () async {
                          final url = urlController.text.trim();
                          final identity = identityController.text.trim();
                          final pass = passwordController.text;

                          if (url.isEmpty || identity.isEmpty || pass.isEmpty) {
                            setState(() {
                              errorMessage = 'All fields are required.';
                            });
                            return;
                          }

                          setState(() {
                            isSubmitting = true;
                            errorMessage = null;
                          });

                          final navigator = Navigator.of(context);
                          final dialogNav = Navigator.of(dialogCtx);
                          final success = await ref
                              .read(authProvider.notifier)
                              .login(
                                serverUrl: url,
                                identity: identity,
                                password: pass,
                              );

                          if (success) {
                            if (dialogCtx.mounted) {
                              dialogNav.pop();
                            }
                            if (context.mounted) {
                              navigator.pop(); // close drawer
                            }
                          } else {
                            final err = ref.read(authProvider).error ??
                                'Failed to connect to server.';
                            setState(() {
                              isSubmitting = false;
                              errorMessage = err;
                            });
                          }
                        },
                  child: isSubmitting
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(
                              strokeWidth: 2, color: Colors.white),
                        )
                      : const Text('Connect & Save'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  void _showSettingsDialog(BuildContext context, WidgetRef ref) {
    final user = ref.read(currentUserProvider);
    final serverName = ref.read(serverNameProvider);

    showModalBottomSheet<void>(
      context: context,
      backgroundColor: SharkordColors.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      builder: (sheetCtx) {
        return SafeArea(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  children: [
                    CircleAvatar(
                      radius: 24,
                      backgroundColor: SharkordColors.card,
                      child: Text(
                        (user?.displayName.isNotEmpty ?? false)
                            ? user!.displayName[0].toUpperCase()
                            : '?',
                        style: const TextStyle(
                          color: SharkordColors.textPrimary,
                          fontSize: 20,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                    const SizedBox(width: 14),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            user?.displayName ?? 'User',
                            style: const TextStyle(
                              color: SharkordColors.textPrimary,
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          Text(
                            '@${user?.username ?? 'anonymous'} • Connected to $serverName',
                            style: const TextStyle(
                              color: SharkordColors.textMuted,
                              fontSize: 12,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 24),
                OutlinedButton.icon(
                  style: OutlinedButton.styleFrom(
                    foregroundColor: SharkordColors.danger,
                    side: const BorderSide(color: SharkordColors.danger),
                    padding: const EdgeInsets.symmetric(vertical: 12),
                  ),
                  icon: const Icon(Icons.logout_rounded, size: 20),
                  label: const Text('Log Out from Server'),
                  onPressed: () {
                    Navigator.of(sheetCtx).pop();
                    Navigator.of(context).pop(); // Close drawer
                    ref.read(authProvider.notifier).logout(removeFromVault: true);
                  },
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
