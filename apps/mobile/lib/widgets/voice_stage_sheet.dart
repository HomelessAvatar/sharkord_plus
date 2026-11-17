import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';

/// Modal bottom sheet voice stage showing active participants and controls.
class VoiceStageSheet extends ConsumerWidget {
  const VoiceStageSheet({super.key});

  /// Presents the voice stage sheet as a modal bottom sheet.
  static Future<void> show(BuildContext context) {
    return showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => const VoiceStageSheet(),
    );
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final voiceState = ref.watch(voiceProvider);
    final currentUser = ref.watch(currentUserProvider);

    // Ensure current user is in participants list if not already
    final participants = List<User>.from(voiceState.participants);
    if (currentUser != null && !participants.any((u) => u.id == currentUser.id)) {
      participants.insert(0, currentUser);
    }

    return Container(
      height: MediaQuery.of(context).size.height * 0.88,
      decoration: const BoxDecoration(
        color: SharkordColors.background,
        borderRadius: BorderRadius.only(
          topLeft: Radius.circular(20),
          topRight: Radius.circular(20),
        ),
      ),
      child: SafeArea(
        child: Column(
          children: [
            const SizedBox(height: 10),
            // Drag Handle
            Center(
              child: Container(
                width: 40,
                height: 4,
                decoration: BoxDecoration(
                  color: SharkordColors.border,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
            const SizedBox(height: 12),

            // Sheet Header
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                children: [
                  IconButton(
                    icon: const Icon(
                      Icons.keyboard_arrow_down_rounded,
                      color: SharkordColors.textPrimary,
                      size: 28,
                    ),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          voiceState.channelName ?? 'Voice Stage',
                          style: const TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.bold,
                            color: SharkordColors.textPrimary,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        Row(
                          children: [
                            const Icon(
                              Icons.wifi_rounded,
                              size: 14,
                              color: SharkordColors.speaking,
                            ),
                            const SizedBox(width: 4),
                            Text(
                              'Connected • ${voiceState.pingMs}ms',
                              style: const TextStyle(
                                fontSize: 12,
                                color: SharkordColors.speaking,
                                fontWeight: FontWeight.w500,
                              ),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 12),
            const Divider(color: SharkordColors.border, height: 1),

            // Stage Body
            Expanded(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    // Video / Screen Share Tile Placeholder
                    Container(
                      height: 140,
                      decoration: BoxDecoration(
                        color: SharkordColors.surface,
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(color: SharkordColors.border),
                      ),
                      child: Center(
                        child: Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Container(
                              width: 48,
                              height: 48,
                              decoration: BoxDecoration(
                                color: SharkordColors.card,
                                shape: BoxShape.circle,
                                border: Border.all(color: SharkordColors.border),
                              ),
                              child: const Icon(
                                Icons.videocam_off_rounded,
                                color: SharkordColors.textMuted,
                                size: 24,
                              ),
                            ),
                            const SizedBox(height: 10),
                            const Text(
                              'Screen Share / Video Tile',
                              style: TextStyle(
                                color: SharkordColors.textPrimary,
                                fontWeight: FontWeight.w600,
                                fontSize: 13,
                              ),
                            ),
                            const SizedBox(height: 4),
                            const Text(
                              'No active video stream',
                              style: TextStyle(
                                color: SharkordColors.textMuted,
                                fontSize: 11,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                    const SizedBox(height: 20),

                    // Participants Header
                    Text(
                      'PARTICIPANTS (${participants.length})',
                      style: const TextStyle(
                        color: SharkordColors.textMuted,
                        fontSize: 12,
                        fontWeight: FontWeight.w700,
                        letterSpacing: 0.8,
                      ),
                    ),
                    const SizedBox(height: 12),

                    // Participants Grid
                    GridView.builder(
                      shrinkWrap: true,
                      physics: const NeverScrollableScrollPhysics(),
                      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: 2,
                        crossAxisSpacing: 12,
                        mainAxisSpacing: 12,
                        childAspectRatio: 1.1,
                      ),
                      itemCount: participants.length,
                      itemBuilder: (context, index) {
                        final user = participants[index];
                        final isSpeaking = voiceState.isSpeaking(user.id);
                        final isSelf = currentUser?.id == user.id;

                        return Container(
                          padding: const EdgeInsets.all(12),
                          decoration: BoxDecoration(
                            color: SharkordColors.card,
                            borderRadius: BorderRadius.circular(12),
                            border: Border.all(
                              color: isSpeaking
                                  ? SharkordColors.speaking
                                  : SharkordColors.border,
                              width: isSpeaking ? 2.5 : 1.0,
                            ),
                            boxShadow: isSpeaking
                                ? [
                                    BoxShadow(
                                      color: SharkordColors.speaking.withAlpha(90),
                                      blurRadius: 10,
                                      spreadRadius: 1,
                                    ),
                                  ]
                                : null,
                          ),
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Stack(
                                clipBehavior: Clip.none,
                                children: [
                                  CircleAvatar(
                                    radius: 26,
                                    backgroundColor: SharkordColors.surface,
                                    child: Text(
                                      user.displayName.isNotEmpty
                                          ? user.displayName[0].toUpperCase()
                                          : 'U',
                                      style: const TextStyle(
                                        color: SharkordColors.textPrimary,
                                        fontWeight: FontWeight.bold,
                                        fontSize: 18,
                                      ),
                                    ),
                                  ),
                                  if (isSelf && voiceState.isMuted)
                                    Positioned(
                                      right: -2,
                                      bottom: -2,
                                      child: Container(
                                        padding: const EdgeInsets.all(3),
                                        decoration: const BoxDecoration(
                                          color: SharkordColors.danger,
                                          shape: BoxShape.circle,
                                        ),
                                        child: const Icon(
                                          Icons.mic_off_rounded,
                                          color: Colors.white,
                                          size: 12,
                                        ),
                                      ),
                                    ),
                                ],
                              ),
                              const SizedBox(height: 10),
                              Text(
                                user.displayName,
                                style: const TextStyle(
                                  color: SharkordColors.textPrimary,
                                  fontWeight: FontWeight.w600,
                                  fontSize: 13,
                                ),
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                              if (isSpeaking)
                                const Text(
                                  'Speaking...',
                                  style: TextStyle(
                                    color: SharkordColors.speaking,
                                    fontSize: 11,
                                    fontWeight: FontWeight.w500,
                                  ),
                                ),
                            ],
                          ),
                        );
                      },
                    ),
                  ],
                ),
              ),
            ),

            // Large Control Actions Bar
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
              decoration: const BoxDecoration(
                color: SharkordColors.surface,
                border: Border(
                  top: BorderSide(color: SharkordColors.border),
                ),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  // Mute Button
                  _buildCircleButton(
                    key: const Key('stage_mute_button'),
                    icon: voiceState.isMuted
                        ? Icons.mic_off_rounded
                        : Icons.mic_rounded,
                    isActive: voiceState.isMuted,
                    activeColor: SharkordColors.danger,
                    inactiveColor: SharkordColors.card,
                    iconColor: voiceState.isMuted
                        ? Colors.white
                        : SharkordColors.textPrimary,
                    onTap: () {
                      ref.read(voiceProvider.notifier).toggleMute();
                    },
                  ),

                  // Deafen Button
                  _buildCircleButton(
                    key: const Key('stage_deafen_button'),
                    icon: voiceState.isDeafened
                        ? Icons.headset_off_rounded
                        : Icons.headset_rounded,
                    isActive: voiceState.isDeafened,
                    activeColor: SharkordColors.danger,
                    inactiveColor: SharkordColors.card,
                    iconColor: voiceState.isDeafened
                        ? Colors.white
                        : SharkordColors.textPrimary,
                    onTap: () {
                      ref.read(voiceProvider.notifier).toggleDeafen();
                    },
                  ),

                  // Disconnect Button
                  _buildCircleButton(
                    key: const Key('stage_disconnect_button'),
                    icon: Icons.call_end_rounded,
                    isActive: true,
                    activeColor: SharkordColors.danger,
                    inactiveColor: SharkordColors.danger,
                    iconColor: Colors.white,
                    onTap: () {
                      ref.read(voiceProvider.notifier).leaveVoice();
                      ref
                          .read(channelProvider.notifier)
                          .selectVoiceChannel(null);
                      Navigator.of(context).pop();
                    },
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCircleButton({
    required Key key,
    required IconData icon,
    required bool isActive,
    required Color activeColor,
    required Color inactiveColor,
    required Color iconColor,
    required VoidCallback onTap,
  }) {
    return GestureDetector(
      key: key,
      onTap: onTap,
      child: Container(
        width: 56,
        height: 56,
        decoration: BoxDecoration(
          color: isActive ? activeColor : inactiveColor,
          shape: BoxShape.circle,
          border: Border.all(
            color: isActive ? activeColor : SharkordColors.border,
          ),
        ),
        child: Icon(
          icon,
          color: iconColor,
          size: 26,
        ),
      ),
    );
  }
}
