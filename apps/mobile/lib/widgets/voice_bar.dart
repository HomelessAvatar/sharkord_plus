import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';
import 'package:sharkord_mobile/widgets/voice_stage_sheet.dart';

/// Docked mini voice bar displayed whenever connected to an active voice channel.
class VoiceBar extends ConsumerWidget {
  const VoiceBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final voiceState = ref.watch(voiceProvider);

    if (!voiceState.isConnected) {
      return const SizedBox.shrink();
    }

    return Container(
      decoration: const BoxDecoration(
        color: SharkordColors.surface,
        border: Border(
          top: BorderSide(color: SharkordColors.border),
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.black26,
            blurRadius: 8,
            offset: Offset(0, -2),
          ),
        ],
      ),
      child: SafeArea(
        top: false,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Row(
            children: [
              // Tap Target for Voice Stage Sheet
              Expanded(
                child: InkWell(
                  key: const Key('voice_bar_tap_target'),
                  borderRadius: BorderRadius.circular(8),
                  onTap: () => VoiceStageSheet.show(context),
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      vertical: 4,
                      horizontal: 4,
                    ),
                    child: Row(
                      children: [
                        // Speaking / Voice Connected Indicator Icon
                        Container(
                          width: 36,
                          height: 36,
                          decoration: BoxDecoration(
                            color: SharkordColors.speaking.withAlpha(38),
                            shape: BoxShape.circle,
                          ),
                          child: const Icon(
                            Icons.volume_up_rounded,
                            color: SharkordColors.speaking,
                            size: 20,
                          ),
                        ),
                        const SizedBox(width: 10),
                        // Channel & Latency Info
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              Text(
                                voiceState.channelName ?? 'Voice Channel',
                                style: const TextStyle(
                                  color: SharkordColors.textPrimary,
                                  fontWeight: FontWeight.bold,
                                  fontSize: 14,
                                ),
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                              Row(
                                children: [
                                  const Icon(
                                    Icons.wifi_rounded,
                                    size: 12,
                                    color: SharkordColors.speaking,
                                  ),
                                  const SizedBox(width: 4),
                                  Text(
                                    'Connected / ${voiceState.pingMs}ms',
                                    style: const TextStyle(
                                      color: SharkordColors.speaking,
                                      fontSize: 11,
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
                ),
              ),

              // Control Buttons
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  // Mute Button
                  IconButton(
                    key: const Key('voice_mute_button'),
                    icon: Icon(
                      voiceState.isMuted
                          ? Icons.mic_off_rounded
                          : Icons.mic_rounded,
                      color: voiceState.isMuted
                          ? SharkordColors.danger
                          : SharkordColors.textPrimary,
                      size: 22,
                    ),
                    onPressed: () {
                      ref.read(voiceProvider.notifier).toggleMute();
                    },
                  ),

                  // Deafen Button
                  IconButton(
                    key: const Key('voice_deafen_button'),
                    icon: Icon(
                      voiceState.isDeafened
                          ? Icons.headset_off_rounded
                          : Icons.headset_rounded,
                      color: voiceState.isDeafened
                          ? SharkordColors.danger
                          : SharkordColors.textPrimary,
                      size: 22,
                    ),
                    onPressed: () {
                      ref.read(voiceProvider.notifier).toggleDeafen();
                    },
                  ),

                  // Disconnect Button
                  IconButton(
                    key: const Key('voice_disconnect_button'),
                    icon: const Icon(
                      Icons.call_end_rounded,
                      color: SharkordColors.danger,
                      size: 22,
                    ),
                    onPressed: () {
                      ref.read(voiceProvider.notifier).leaveVoice();
                      ref
                          .read(channelProvider.notifier)
                          .selectVoiceChannel(null);
                    },
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
