import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/auth_provider.dart';
import 'package:sharkord_mobile/providers/member_provider.dart';

/// Right Drawer displaying categorized Online and Offline member rosters.
class MemberDrawer extends ConsumerWidget {
  const MemberDrawer({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final onlineMembers = ref.watch(onlineMembersProvider);
    final offlineMembers = ref.watch(offlineMembersProvider);
    final currentUser = ref.watch(currentUserProvider);

    // Merge current user into online list if not already present
    final mergedOnline = List<User>.from(onlineMembers);
    if (currentUser != null && !mergedOnline.any((m) => m.id == currentUser.id)) {
      mergedOnline.insert(0, currentUser.copyWith(isOnline: true));
    }

    return Drawer(
      backgroundColor: SharkordColors.sidebar,
      child: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Drawer Header
            Container(
              height: 56,
              padding: const EdgeInsets.symmetric(horizontal: 16),
              decoration: const BoxDecoration(
                border: Border(
                  bottom: BorderSide(color: SharkordColors.border),
                ),
              ),
              child: const Row(
                children: [
                  Icon(
                    Icons.people_alt_rounded,
                    color: SharkordColors.textPrimary,
                    size: 20,
                  ),
                  SizedBox(width: 10),
                  Text(
                    'Members',
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                      color: SharkordColors.textPrimary,
                    ),
                  ),
                ],
              ),
            ),

            // Members List
            Expanded(
              child: ListView(
                padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 8),
                children: [
                  // Online Section
                  _buildSectionHeader('ONLINE — ${mergedOnline.length}'),
                  ...mergedOnline.map((user) => _buildMemberTile(user: user, isOnline: true)),
                  const SizedBox(height: 16),

                  // Offline Section
                  _buildSectionHeader('OFFLINE — ${offlineMembers.length}'),
                  ...offlineMembers.map((user) => _buildMemberTile(user: user, isOnline: false)),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
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

  Widget _buildMemberTile({required User user, required bool isOnline}) {
    final initial = user.displayName.isNotEmpty
        ? user.displayName[0].toUpperCase()
        : user.username.isNotEmpty
            ? user.username[0].toUpperCase()
            : 'U';

    return Opacity(
      opacity: isOnline ? 1.0 : 0.6,
      child: Container(
        margin: const EdgeInsets.symmetric(vertical: 2),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(6),
        ),
        child: Row(
          children: [
            // Avatar with Status Dot Indicator
            Stack(
              clipBehavior: Clip.none,
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
                Positioned(
                  right: -1,
                  bottom: -1,
                  child: Container(
                    width: 11,
                    height: 11,
                    decoration: BoxDecoration(
                      color: isOnline
                          ? SharkordColors.speaking
                          : SharkordColors.textMuted,
                      shape: BoxShape.circle,
                      border: Border.all(
                        color: SharkordColors.sidebar,
                        width: 2,
                      ),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(width: 12),

            // Display Name and Subtitle
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    user.displayName,
                    style: TextStyle(
                      color: isOnline
                          ? SharkordColors.textPrimary
                          : SharkordColors.textMuted,
                      fontWeight: FontWeight.w600,
                      fontSize: 14,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  Text(
                    '@${user.username}',
                    style: const TextStyle(
                      color: SharkordColors.textMuted,
                      fontSize: 11,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
