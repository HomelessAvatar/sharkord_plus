import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/models/channel.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';

/// Left Drawer featuring Server Rail and categorized Channel Tree.
class ChannelDrawer extends ConsumerWidget {
  const ChannelDrawer({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final channelState = ref.watch(channelProvider);
    final voiceState = ref.watch(voiceProvider);
    final currentUser = ref.watch(currentUserProvider);

    // Group channels by category
    final textChannels = channelState.textChannels;
    final voiceChannels = channelState.voiceChannels;

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
                  // Active Server Pill & Icon
                  Row(
                    children: [
                      // Active indicator pill
                      Container(
                        width: 4,
                        height: 36,
                        decoration: const BoxDecoration(
                          color: SharkordColors.primary,
                          borderRadius: BorderRadius.only(
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
                          borderRadius: BorderRadius.circular(14),
                          border: Border.all(
                            color: SharkordColors.primary.withAlpha(128),
                            width: 1.5,
                          ),
                        ),
                        child: const Center(
                          child: Icon(
                            Icons.waves_rounded,
                            color: SharkordColors.primary,
                            size: 26,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  const Divider(
                    color: SharkordColors.border,
                    indent: 14,
                    endIndent: 14,
                  ),
                  const SizedBox(height: 8),
                  // Add Server Action
                  Container(
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
                  const Spacer(),
                  // Settings / Profile
                  Container(
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
                        const Expanded(
                          child: Text(
                            'Sharkord Server',
                            style: TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                              color: SharkordColors.textPrimary,
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        Icon(
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
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
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
              Navigator.of(context).pop();
            }
          },
        ),
      ),
    );
  }
}
