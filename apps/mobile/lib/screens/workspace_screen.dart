import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/providers/channel_provider.dart';
import 'package:sharkord_mobile/providers/update_provider.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';
import 'package:sharkord_mobile/widgets/channel_drawer.dart';
import 'package:sharkord_mobile/widgets/chat_canvas.dart';
import 'package:sharkord_mobile/widgets/member_drawer.dart';
import 'package:sharkord_mobile/widgets/update_dialog.dart';
import 'package:sharkord_mobile/widgets/voice_bar.dart';

/// Main workspace layout housing three-pane navigation, chat canvas, and voice bar.
class WorkspaceScreen extends ConsumerWidget {
  const WorkspaceScreen({super.key});

  Future<void> _handleCheckUpdate(BuildContext context, WidgetRef ref) async {
    final result =
        await ref.read(updateProvider.notifier).checkForUpdate();

    if (context.mounted) {
      if (result != null && result.hasUpdate) {
        UpdateDialog.show(context, result);
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Sharkord Plus is up to date!'),
            duration: Duration(seconds: 2),
          ),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selectedChannel = ref.watch(selectedChannelProvider);
    final channelState = ref.watch(channelProvider);
    final isVoiceConnected = ref.watch(isVoiceConnectedProvider);
    final channelName = selectedChannel?.name ??
        (channelState.channels.isEmpty ? 'No Channel' : 'general');

    return Scaffold(
      backgroundColor: SharkordColors.background,
      drawer: const ChannelDrawer(),
      endDrawer: const MemberDrawer(),
      appBar: AppBar(
        backgroundColor: SharkordColors.surface,
        elevation: 0,
        leading: Builder(
          builder: (context) => IconButton(
            key: const Key('open_drawer_button'),
            icon: const Icon(
              Icons.menu_rounded,
              color: SharkordColors.textPrimary,
            ),
            onPressed: () => Scaffold.of(context).openDrawer(),
          ),
        ),
        title: Row(
          children: [
            const Icon(
              Icons.tag_rounded,
              color: SharkordColors.textMuted,
              size: 20,
            ),
            const SizedBox(width: 6),
            Expanded(
              child: Text(
                channelName,
                style: const TextStyle(
                  fontSize: 17,
                  fontWeight: FontWeight.bold,
                  color: SharkordColors.textPrimary,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
        actions: [
          // Update Check Action
          IconButton(
            key: const Key('check_update_button'),
            icon: const Icon(
              Icons.system_update_rounded,
              color: SharkordColors.textMuted,
              size: 22,
            ),
            tooltip: 'Check for Updates',
            onPressed: () => _handleCheckUpdate(context, ref),
          ),

          // Member Drawer Action
          Builder(
            builder: (context) => IconButton(
              key: const Key('open_members_button'),
              icon: const Icon(
                Icons.people_alt_rounded,
                color: SharkordColors.textMuted,
                size: 22,
              ),
              tooltip: 'Members',
              onPressed: () => Scaffold.of(context).openEndDrawer(),
            ),
          ),
        ],
      ),
      body: Column(
        children: [
          // Main Chat Feed
          const Expanded(
            child: ChatCanvas(),
          ),

          // Docked Voice Bar when connected
          if (isVoiceConnected) const VoiceBar(),
        ],
      ),
    );
  }
}
