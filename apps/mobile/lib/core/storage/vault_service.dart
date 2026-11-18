import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Representation of an authenticated Sharkord server session.
@immutable
class SavedServerSession {
  final String serverId;
  final String serverName;
  final String serverUrl;
  final String token;
  final String userId;
  final String username;
  final String displayName;
  final String? avatarUrl;
  final DateTime? lastConnectedAt;

  const SavedServerSession({
    required this.serverId,
    required this.serverName,
    required this.serverUrl,
    required this.token,
    required this.userId,
    required this.username,
    required this.displayName,
    this.avatarUrl,
    this.lastConnectedAt,
  });

  Map<String, dynamic> toJson() {
    return {
      'serverId': serverId,
      'serverName': serverName,
      'serverUrl': serverUrl,
      'token': token,
      'userId': userId,
      'username': username,
      'displayName': displayName,
      if (avatarUrl != null) 'avatarUrl': avatarUrl,
      if (lastConnectedAt != null)
        'lastConnectedAt': lastConnectedAt!.toIso8601String(),
    };
  }

  factory SavedServerSession.fromJson(Map<String, dynamic> json) {
    return SavedServerSession(
      serverId: (json['serverId'] ?? '').toString(),
      serverName: (json['serverName'] ?? 'Sharkord Server').toString(),
      serverUrl: (json['serverUrl'] ?? '').toString(),
      token: (json['token'] ?? '').toString(),
      userId: (json['userId'] ?? '').toString(),
      username: (json['username'] ?? '').toString(),
      displayName: (json['displayName'] ?? json['username'] ?? 'User').toString(),
      avatarUrl: json['avatarUrl'] as String?,
      lastConnectedAt: json['lastConnectedAt'] != null
          ? DateTime.tryParse(json['lastConnectedAt'].toString())
          : null,
    );
  }

  SavedServerSession copyWith({
    String? serverId,
    String? serverName,
    String? serverUrl,
    String? token,
    String? userId,
    String? username,
    String? displayName,
    String? avatarUrl,
    DateTime? lastConnectedAt,
  }) {
    return SavedServerSession(
      serverId: serverId ?? this.serverId,
      serverName: serverName ?? this.serverName,
      serverUrl: serverUrl ?? this.serverUrl,
      token: token ?? this.token,
      userId: userId ?? this.userId,
      username: username ?? this.username,
      displayName: displayName ?? this.displayName,
      avatarUrl: avatarUrl ?? this.avatarUrl,
      lastConnectedAt: lastConnectedAt ?? this.lastConnectedAt,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SavedServerSession &&
          runtimeType == other.runtimeType &&
          serverId == other.serverId &&
          serverUrl == other.serverUrl &&
          token == other.token &&
          userId == other.userId;

  @override
  int get hashCode => Object.hash(serverId, serverUrl, token, userId);
}

/// Service managing persistent storage of server sessions and active server pointer.
class VaultService {
  static const String _serversKey = 'sharkord_vault_servers';
  static const String _activeServerIdKey = 'sharkord_vault_active_server_id';

  final SharedPreferences? _prefs;

  VaultService({SharedPreferences? prefs}) : _prefs = prefs;

  Future<SharedPreferences> _getPrefs() async {
    return _prefs ?? await SharedPreferences.getInstance();
  }

  /// Retrieves all saved server sessions stored in the vault.
  Future<List<SavedServerSession>> getSavedServers() async {
    try {
      final prefs = await _getPrefs();
      final raw = prefs.getString(_serversKey);
      if (raw == null || raw.isEmpty) {
        return const [];
      }
      final decoded = jsonDecode(raw);
      if (decoded is List) {
        return decoded
            .map((item) =>
                SavedServerSession.fromJson(Map<String, dynamic>.from(item as Map)))
            .toList();
      }
      return const [];
    } catch (_) {
      return const [];
    }
  }

  /// Returns the currently active server session, or the first available saved server.
  Future<SavedServerSession?> getActiveServer() async {
    try {
      final prefs = await _getPrefs();
      final activeId = prefs.getString(_activeServerIdKey);
      final servers = await getSavedServers();
      if (servers.isEmpty) return null;

      if (activeId != null) {
        final match = servers.where((s) => s.serverId == activeId);
        if (match.isNotEmpty) return match.first;
      }
      return servers.first;
    } catch (_) {
      return null;
    }
  }

  /// Returns the ID of the active server.
  Future<String?> getActiveServerId() async {
    try {
      final prefs = await _getPrefs();
      return prefs.getString(_activeServerIdKey);
    } catch (_) {
      return null;
    }
  }

  /// Saves or updates a server session in persistent storage.
  Future<void> saveServer(SavedServerSession server, {bool setAsActive = true}) async {
    try {
      final prefs = await _getPrefs();
      final servers = (await getSavedServers()).toList();

      final existingIndex = servers.indexWhere(
        (s) => s.serverId == server.serverId || s.serverUrl == server.serverUrl,
      );

      if (existingIndex >= 0) {
        servers[existingIndex] = server;
      } else {
        servers.add(server);
      }

      final encoded = jsonEncode(servers.map((s) => s.toJson()).toList());
      await prefs.setString(_serversKey, encoded);

      if (setAsActive) {
        await prefs.setString(_activeServerIdKey, server.serverId);
      }
    } catch (_) {}
  }

  /// Sets the active server ID in persistent storage.
  Future<void> setActiveServerId(String serverId) async {
    try {
      final prefs = await _getPrefs();
      await prefs.setString(_activeServerIdKey, serverId);
    } catch (_) {}
  }

  /// Removes a server session from the vault.
  Future<void> removeServer(String serverId) async {
    try {
      final prefs = await _getPrefs();
      final servers = (await getSavedServers()).toList();
      servers.removeWhere((s) => s.serverId == serverId);

      final encoded = jsonEncode(servers.map((s) => s.toJson()).toList());
      await prefs.setString(_serversKey, encoded);

      final currentActiveId = prefs.getString(_activeServerIdKey);
      if (currentActiveId == serverId) {
        if (servers.isNotEmpty) {
          await prefs.setString(_activeServerIdKey, servers.first.serverId);
        } else {
          await prefs.remove(_activeServerIdKey);
        }
      }
    } catch (_) {}
  }

  /// Clears all saved server sessions and active selection.
  Future<void> clearAll() async {
    try {
      final prefs = await _getPrefs();
      await prefs.remove(_serversKey);
      await prefs.remove(_activeServerIdKey);
    } catch (_) {}
  }
}

/// Provider for VaultService instance.
final vaultServiceProvider = Provider<VaultService>((ref) {
  return VaultService();
});

/// Notifier managing reactive list of saved servers in state.
class SavedServersNotifier extends StateNotifier<List<SavedServerSession>> {
  final VaultService _vault;

  SavedServersNotifier(this._vault) : super(const []) {
    loadServers();
  }

  Future<void> loadServers() async {
    final servers = await _vault.getSavedServers();
    if (!mounted) return;
    state = servers;
  }

  Future<void> addOrUpdate(SavedServerSession server, {bool setAsActive = true}) async {
    await _vault.saveServer(server, setAsActive: setAsActive);
    if (!mounted) return;
    await loadServers();
  }

  Future<void> remove(String serverId) async {
    await _vault.removeServer(serverId);
    if (!mounted) return;
    await loadServers();
  }
}

/// Provider for reactive list of saved servers.
final savedServersProvider =
    StateNotifierProvider<SavedServersNotifier, List<SavedServerSession>>((ref) {
  final vault = ref.watch(vaultServiceProvider);
  return SavedServersNotifier(vault);
});
