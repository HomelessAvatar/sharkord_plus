import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:sharkord_mobile/core/update/update_model.dart';
import 'package:sharkord_mobile/core/update/update_service.dart';

/// State of application updates.
@immutable
class UpdateState {
  final bool isChecking;
  final bool isDownloading;
  final double downloadProgress;
  final UpdateCheckResult? updateResult;
  final String? errorMessage;

  const UpdateState({
    this.isChecking = false,
    this.isDownloading = false,
    this.downloadProgress = 0.0,
    this.updateResult,
    this.errorMessage,
  });

  /// True if an update is available.
  bool get hasUpdate => updateResult != null && updateResult!.hasUpdate;

  UpdateState copyWith({
    bool? isChecking,
    bool? isDownloading,
    double? downloadProgress,
    UpdateCheckResult? updateResult,
    String? errorMessage,
    bool clearResult = false,
    bool clearError = false,
  }) {
    return UpdateState(
      isChecking: isChecking ?? this.isChecking,
      isDownloading: isDownloading ?? this.isDownloading,
      downloadProgress: downloadProgress ?? this.downloadProgress,
      updateResult: clearResult ? null : (updateResult ?? this.updateResult),
      errorMessage: clearError ? null : (errorMessage ?? this.errorMessage),
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is UpdateState &&
          runtimeType == other.runtimeType &&
          isChecking == other.isChecking &&
          isDownloading == other.isDownloading &&
          downloadProgress == other.downloadProgress &&
          updateResult == other.updateResult &&
          errorMessage == other.errorMessage;

  @override
  int get hashCode => Object.hash(
        isChecking,
        isDownloading,
        downloadProgress,
        updateResult,
        errorMessage,
      );
}

/// Notifier coordinating update checks and download status.
class UpdateNotifier extends StateNotifier<UpdateState> {
  final UpdateService _updateService;

  UpdateNotifier([UpdateService? updateService])
      : _updateService = updateService ?? UpdateService(),
        super(const UpdateState());

  /// Checks for application updates from GitHub releases.
  Future<UpdateCheckResult?> checkForUpdate({
    bool silent = false,
    String? currentVersionOverride,
    bool? isAndroidOverride,
  }) async {
    state = state.copyWith(isChecking: true, clearError: true);

    try {
      final result = await _updateService.checkForUpdate(
        currentVersionOverride: currentVersionOverride,
        isAndroidOverride: isAndroidOverride,
      );

      state = state.copyWith(
        isChecking: false,
        updateResult: result,
        clearError: true,
      );
      return result;
    } catch (e) {
      state = state.copyWith(
        isChecking: false,
        errorMessage: e.toString(),
      );
      return null;
    }
  }

  /// Updates streaming APK download progress (0.0 to 1.0).
  void setDownloadProgress(double progress) {
    state = state.copyWith(
      downloadProgress: progress.clamp(0.0, 1.0),
      isDownloading: progress < 1.0,
    );
  }

  /// Sets downloading status.
  void setDownloading(bool downloading) {
    state = state.copyWith(isDownloading: downloading);
  }

  /// Resets update state to default.
  void reset() {
    state = const UpdateState();
  }
}

/// Provider for UpdateService singleton.
final updateServiceProvider = Provider<UpdateService>((ref) {
  return UpdateService();
});

/// Provider for update state and actions.
final updateProvider =
    StateNotifierProvider<UpdateNotifier, UpdateState>((ref) {
  final service = ref.watch(updateServiceProvider);
  return UpdateNotifier(service);
});

/// Convenience provider checking if an update is available.
final hasUpdateProvider = Provider<bool>((ref) {
  return ref.watch(updateProvider).hasUpdate;
});
