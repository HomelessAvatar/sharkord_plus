import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/models/user.dart';

/// State of authentication in the mobile client.
@immutable
class AuthState {
  final User? user;
  final String? token;
  final String? serverUrl;
  final bool isLoading;
  final String? error;

  const AuthState({
    this.user,
    this.token,
    this.serverUrl,
    this.isLoading = false,
    this.error,
  });

  /// True if user is authenticated with a valid token.
  bool get isAuthenticated => user != null && token != null;

  AuthState copyWith({
    User? user,
    String? token,
    String? serverUrl,
    bool? isLoading,
    String? error,
    bool clearUser = false,
    bool clearToken = false,
    bool clearError = false,
  }) {
    return AuthState(
      user: clearUser ? null : (user ?? this.user),
      token: clearToken ? null : (token ?? this.token),
      serverUrl: serverUrl ?? this.serverUrl,
      isLoading: isLoading ?? this.isLoading,
      error: clearError ? null : (error ?? this.error),
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is AuthState &&
          runtimeType == other.runtimeType &&
          user == other.user &&
          token == other.token &&
          serverUrl == other.serverUrl &&
          isLoading == other.isLoading &&
          error == other.error;

  @override
  int get hashCode => Object.hash(user, token, serverUrl, isLoading, error);
}

/// Notifier managing authentication state and actions.
class AuthNotifier extends StateNotifier<AuthState> {
  AuthNotifier() : super(const AuthState());

  /// Authenticates with a Sharkord server instance.
  Future<bool> login({
    required String serverUrl,
    required String identity,
    required String password,
  }) async {
    final cleanUrl = serverUrl.trim();
    final cleanIdentity = identity.trim();

    if (cleanUrl.isEmpty) {
      state = state.copyWith(error: 'Server URL cannot be empty');
      return false;
    }
    if (cleanIdentity.isEmpty) {
      state = state.copyWith(error: 'Username or email cannot be empty');
      return false;
    }
    if (password.isEmpty) {
      state = state.copyWith(error: 'Password cannot be empty');
      return false;
    }

    state = state.copyWith(isLoading: true, clearError: true);

    try {
      // Simulate network authentication round-trip or FFI bridge invocation
      await Future<void>.delayed(const Duration(milliseconds: 10));

      final token = 'auth_token_${DateTime.now().millisecondsSinceEpoch}';
      final user = User(
        id: '1',
        username: cleanIdentity,
        displayName: cleanIdentity,
        avatarUrl: null,
        isOnline: true,
      );

      state = state.copyWith(
        user: user,
        token: token,
        serverUrl: cleanUrl,
        isLoading: false,
        clearError: true,
      );
      return true;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: e.toString(),
      );
      return false;
    }
  }

  /// Sets an active session directly (useful for tests and token restore).
  void setSession({
    required User user,
    required String token,
    required String serverUrl,
  }) {
    state = state.copyWith(
      user: user,
      token: token,
      serverUrl: serverUrl,
      isLoading: false,
      clearError: true,
    );
  }

  /// Logs out the user and clears all session tokens.
  void logout() {
    state = const AuthState();
  }

  /// Clears active error message.
  void clearError() {
    state = state.copyWith(clearError: true);
  }
}

/// Provider for authentication state.
final authProvider = StateNotifierProvider<AuthNotifier, AuthState>((ref) {
  return AuthNotifier();
});

/// Convenience provider for current authenticated user.
final currentUserProvider = Provider<User?>((ref) {
  return ref.watch(authProvider).user;
});

/// Convenience provider checking if user is logged in.
final isAuthenticatedProvider = Provider<bool>((ref) {
  return ref.watch(authProvider).isAuthenticated;
});
