import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Server state representing connected server metadata.
@immutable
class ServerState {
  final String serverId;
  final String serverName;
  final String? description;
  final String? logoUrl;

  const ServerState({
    this.serverId = '',
    this.serverName = 'Sharkord Server',
    this.description,
    this.logoUrl,
  });

  ServerState copyWith({
    String? serverId,
    String? serverName,
    String? description,
    String? logoUrl,
  }) {
    return ServerState(
      serverId: serverId ?? this.serverId,
      serverName: serverName ?? this.serverName,
      description: description ?? this.description,
      logoUrl: logoUrl ?? this.logoUrl,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ServerState &&
          runtimeType == other.runtimeType &&
          serverId == other.serverId &&
          serverName == other.serverName &&
          description == other.description &&
          logoUrl == other.logoUrl;

  @override
  int get hashCode => Object.hash(serverId, serverName, description, logoUrl);
}

/// Notifier managing connected server state.
class ServerNotifier extends StateNotifier<ServerState> {
  ServerNotifier([ServerState? initial]) : super(initial ?? const ServerState());

  /// Sets the active server details.
  void setServerDetails({
    required String serverId,
    required String serverName,
    String? description,
    String? logoUrl,
  }) {
    state = state.copyWith(
      serverId: serverId,
      serverName: serverName,
      description: description,
      logoUrl: logoUrl,
    );
  }

  /// Sets just the server name.
  void setServerName(String name) {
    state = state.copyWith(serverName: name);
  }

  /// Resets to default initial state.
  void reset() {
    state = const ServerState();
  }
}

/// Provider for server metadata state.
final serverProvider = StateNotifierProvider<ServerNotifier, ServerState>((ref) {
  return ServerNotifier();
});

/// Convenience provider for current server name.
final serverNameProvider = Provider<String>((ref) {
  return ref.watch(serverProvider).serverName;
});
