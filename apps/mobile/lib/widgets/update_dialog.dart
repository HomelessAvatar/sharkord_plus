import 'package:flutter/material.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/core/update/android_updater.dart';
import 'package:sharkord_mobile/core/update/ios_updater.dart';
import 'package:sharkord_mobile/core/update/update_model.dart';

class UpdateDialog extends StatefulWidget {
  final UpdateCheckResult updateResult;
  final AndroidUpdater? androidUpdater;
  final IosUpdater? iosUpdater;
  final VoidCallback? onDismiss;
  final Future<void> Function()? onInstallCustom;

  const UpdateDialog({
    super.key,
    required this.updateResult,
    this.androidUpdater,
    this.iosUpdater,
    this.onDismiss,
    this.onInstallCustom,
  });

  static Future<void> show(BuildContext context, UpdateCheckResult result) {
    return showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (context) => UpdateDialog(updateResult: result),
    );
  }

  @override
  State<UpdateDialog> createState() => _UpdateDialogState();
}

class _UpdateDialogState extends State<UpdateDialog> {
  bool _isDownloading = false;
  double _progress = 0.0;
  int _receivedBytes = 0;
  int _totalBytes = 0;
  String? _errorMessage;

  late final AndroidUpdater _androidUpdater =
      widget.androidUpdater ?? const AndroidUpdater();
  late final IosUpdater _iosUpdater = widget.iosUpdater ?? const IosUpdater();

  Future<void> _handlePrimaryAction() async {
    if (widget.onInstallCustom != null) {
      await widget.onInstallCustom!();
      return;
    }

    final targetAsset = widget.updateResult.targetAsset;
    final latestRelease = widget.updateResult.latestRelease;

    if (targetAsset != null) {
      setState(() {
        _isDownloading = true;
        _errorMessage = null;
        _progress = 0.0;
      });

      try {
        await _androidUpdater.downloadAndInstallApk(
          targetAsset.downloadUrl,
          onProgress: (progress, received, total) {
            if (mounted) {
              setState(() {
                _progress = progress;
                _receivedBytes = received;
                _totalBytes = total;
              });
            }
          },
        );
      } catch (err) {
        if (mounted) {
          setState(() {
            _errorMessage = err.toString();
            _isDownloading = false;
          });
        }
      }
    } else if (latestRelease != null && latestRelease.htmlUrl.isNotEmpty) {
      await _iosUpdater.openUpdatePage(latestRelease.htmlUrl);
    }
  }

  void _handleDismiss() {
    if (widget.onDismiss != null) {
      widget.onDismiss!();
    } else {
      Navigator.of(context, rootNavigator: true).pop();
    }
  }

  @override
  Widget build(BuildContext context) {
    final release = widget.updateResult.latestRelease;
    final tagName = release?.tagName ?? 'New Version';
    final currentVersion = widget.updateResult.currentVersion;
    final notes = release?.body ?? 'No release notes available.';
    final isAndroidApk = widget.updateResult.targetAsset != null;

    return Dialog(
      backgroundColor: SharkordColors.card,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: const BorderSide(color: SharkordColors.border),
      ),
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: SharkordColors.primary.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: const Icon(
                    Icons.system_update_rounded,
                    color: SharkordColors.primary,
                    size: 24,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        'Update Available',
                        style: TextStyle(
                          color: SharkordColors.textPrimary,
                          fontSize: 18,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Row(
                        children: [
                          Container(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 6,
                              vertical: 2,
                            ),
                            decoration: BoxDecoration(
                              color: SharkordColors.primary.withValues(alpha: 0.2),
                              borderRadius: BorderRadius.circular(6),
                            ),
                            child: Text(
                              tagName,
                              style: const TextStyle(
                                color: SharkordColors.primary,
                                fontSize: 12,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                          ),
                          const SizedBox(width: 8),
                          Text(
                            'Current: $currentVersion',
                            style: const TextStyle(
                              color: SharkordColors.textMuted,
                              fontSize: 12,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            const Text(
              'Release Notes',
              style: TextStyle(
                color: SharkordColors.textPrimary,
                fontSize: 13,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            Container(
              constraints: const BoxConstraints(maxHeight: 160),
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: SharkordColors.surface,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: SharkordColors.border),
              ),
              child: SingleChildScrollView(
                child: Text(
                  notes,
                  style: const TextStyle(
                    color: SharkordColors.textPrimary,
                    fontSize: 13,
                    height: 1.4,
                  ),
                ),
              ),
            ),
            if (_isDownloading) ...[
              const SizedBox(height: 16),
              LinearProgressIndicator(
                value: _progress > 0 ? _progress : null,
                backgroundColor: SharkordColors.surface,
                valueColor: const AlwaysStoppedAnimation<Color>(SharkordColors.primary),
                borderRadius: BorderRadius.circular(4),
              ),
              const SizedBox(height: 6),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    '${(_progress * 100).toInt()}%',
                    style: const TextStyle(
                      color: SharkordColors.textMuted,
                      fontSize: 12,
                    ),
                  ),
                  if (_totalBytes > 0)
                    Text(
                      '${(_receivedBytes / (1024 * 1024)).toStringAsFixed(1)} MB / ${(_totalBytes / (1024 * 1024)).toStringAsFixed(1)} MB',
                      style: const TextStyle(
                        color: SharkordColors.textMuted,
                        fontSize: 12,
                      ),
                    ),
                ],
              ),
            ],
            if (_errorMessage != null) ...[
              const SizedBox(height: 12),
              Text(
                _errorMessage!,
                style: const TextStyle(
                  color: SharkordColors.danger,
                  fontSize: 12,
                ),
              ),
            ],
            const SizedBox(height: 20),
            Row(
              children: [
                Expanded(
                  child: TextButton(
                    onPressed: _isDownloading ? null : _handleDismiss,
                    style: TextButton.styleFrom(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      foregroundColor: SharkordColors.textMuted,
                    ),
                    child: const Text('Later'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  flex: 2,
                  child: ElevatedButton(
                    onPressed: _isDownloading ? null : _handlePrimaryAction,
                    style: ElevatedButton.styleFrom(
                      backgroundColor: SharkordColors.primary,
                      foregroundColor: Colors.white,
                      disabledBackgroundColor: SharkordColors.primary.withValues(alpha: 0.5),
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(8),
                      ),
                    ),
                    child: Text(
                      _isDownloading
                          ? 'Downloading...'
                          : isAndroidApk
                              ? 'Download & Install'
                              : 'Open Update Page',
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
