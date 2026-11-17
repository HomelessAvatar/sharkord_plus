import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import 'package:package_info_plus/package_info_plus.dart';
import 'package:sharkord_mobile/core/update/update_model.dart';

class UpdateService {
  final String repo;
  final http.Client? _defaultClient;

  UpdateService({
    this.repo = 'HomelessAvatar/sharkord_plus',
    http.Client? client,
  }) : _defaultClient = client;

  Future<UpdateCheckResult> checkForUpdate({
    http.Client? client,
    String? currentVersionOverride,
    bool? isAndroidOverride,
  }) async {
    final effectiveClient = client ?? _defaultClient ?? http.Client();
    final shouldCloseClient = client == null && _defaultClient == null;

    try {
      String currentVersion = currentVersionOverride ?? '';
      if (currentVersion.isEmpty) {
        try {
          final packageInfo = await PackageInfo.fromPlatform();
          currentVersion = packageInfo.version;
        } catch (_) {
          currentVersion = '0.0.0';
        }
      }

      final url = Uri.parse('https://api.github.com/repos/$repo/releases/latest');
      final response = await effectiveClient.get(
        url,
        headers: {
          'Accept': 'application/vnd.github.v3+json',
          'User-Agent': 'Sharkord-Mobile-App',
        },
      );

      if (response.statusCode != 200) {
        return UpdateCheckResult(
          hasUpdate: false,
          currentVersion: currentVersion,
        );
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final release = ReleaseInfo.fromJson(json);

      final hasUpdate = SemVer.isNewer(currentVersion, release.tagName);
      final isAndroid = isAndroidOverride ?? (defaultTargetPlatform == TargetPlatform.android);

      ReleaseAsset? targetAsset;
      if (hasUpdate && isAndroid) {
        targetAsset = release.apkAsset;
      }

      return UpdateCheckResult(
        hasUpdate: hasUpdate,
        currentVersion: currentVersion,
        latestRelease: release,
        targetAsset: targetAsset,
      );
    } catch (_) {
      return UpdateCheckResult(
        hasUpdate: false,
        currentVersion: currentVersionOverride ?? '0.0.0',
      );
    } finally {
      if (shouldCloseClient) {
        effectiveClient.close();
      }
    }
  }
}
