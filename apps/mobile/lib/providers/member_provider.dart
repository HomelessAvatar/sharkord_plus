import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/models/user.dart';

/// Notifier managing server members roster and online states.
class MemberNotifier extends StateNotifier<List<User>> {
  MemberNotifier([List<User>? initial]) : super(initial ?? const []);

  /// Overwrites the full member list.
  void setMembers(List<User> members) {
    state = List.unmodifiable(members);
  }

  /// Appends or replaces a member.
  void addOrUpdateMember(User member) {
    final index = state.indexWhere((m) => m.id == member.id);
    if (index >= 0) {
      final updated = List<User>.from(state);
      updated[index] = member;
      state = List.unmodifiable(updated);
    } else {
      state = List.unmodifiable([...state, member]);
    }
  }

  /// Updates online status of a specific member by ID.
  void setOnlineStatus(String userId, bool isOnline) {
    final updated = state.map((m) {
      return m.id == userId ? m.copyWith(isOnline: isOnline) : m;
    }).toList();
    state = List.unmodifiable(updated);
  }

  /// Resets member roster (e.g. on logout).
  void clear() {
    state = const [];
  }
}

/// Provider for server member roster.
final memberProvider =
    StateNotifierProvider<MemberNotifier, List<User>>((ref) {
  return MemberNotifier();
});

/// Filtered provider for currently online members.
final onlineMembersProvider = Provider<List<User>>((ref) {
  final members = ref.watch(memberProvider);
  return members.where((m) => m.isOnline).toList();
});

/// Filtered provider for offline members.
final offlineMembersProvider = Provider<List<User>>((ref) {
  final members = ref.watch(memberProvider);
  return members.where((m) => !m.isOnline).toList();
});
