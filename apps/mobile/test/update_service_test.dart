import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/core/update/android_updater.dart';
import 'package:sharkord_mobile/core/update/ios_updater.dart';
import 'package:sharkord_mobile/core/update/update_model.dart';
import 'package:sharkord_mobile/core/update/update_service.dart';
import 'package:sharkord_mobile/widgets/update_dialog.dart';

void main() {
  group('SemVer Comparison & Parsing', () {
    test('parses standard semantic version strings', () {
      final v1 = SemVer.parse('0.2.0');
      expect(v1.major, 0);
      expect(v1.minor, 2);
      expect(v1.patch, 0);
      expect(v1.preRelease, isNull);

      final v2 = SemVer.parse('v1.3.5');
      expect(v2.major, 1);
      expect(v2.minor, 3);
      expect(v2.patch, 5);

      final v3 = SemVer.parse('V2.0.0-rc.1+build.123');
      expect(v3.major, 2);
      expect(v3.minor, 0);
      expect(v3.patch, 0);
      expect(v3.preRelease, 'rc.1');

      final v4 = SemVer.parse('mobile-v0.2.5');
      expect(v4.major, 0);
      expect(v4.minor, 2);
      expect(v4.patch, 5);

      final v5 = SemVer.parse('desktop-v1.4.2');
      expect(v5.major, 1);
      expect(v5.minor, 4);
      expect(v5.patch, 2);
    });

    test('handles whitespace and partial versions safely', () {
      final v = SemVer.parse('   v3.4   ');
      expect(v.major, 3);
      expect(v.minor, 4);
      expect(v.patch, 0);

      final invalid = SemVer.parse('not-a-version');
      expect(invalid.major, 0);
      expect(invalid.minor, 0);
      expect(invalid.patch, 0);
    });

    test('isNewer compares versions accurately', () {
      expect(SemVer.isNewer('0.2.0', '0.2.1'), isTrue);
      expect(SemVer.isNewer('0.2.0', 'v0.2.1'), isTrue);
      expect(SemVer.isNewer('v0.2.0', '0.2.1'), isTrue);
      expect(SemVer.isNewer('0.9.9', '1.0.0'), isTrue);
      expect(SemVer.isNewer('1.2.0', '1.10.0'), isTrue);
      expect(SemVer.isNewer('0.2.1', '0.2.0'), isFalse);
      expect(SemVer.isNewer('0.2.0', '0.2.0'), isFalse);
      expect(SemVer.isNewer('v1.0.0', '1.0.0'), isFalse);
      expect(SemVer.isNewer('invalid', '1.0.0'), isTrue);
    });

    test('handles pre-release precedence', () {
      expect(SemVer.isNewer('1.0.0-alpha', '1.0.0'), isTrue);
      expect(SemVer.isNewer('1.0.0-alpha', '1.0.0-beta'), isTrue);
      expect(SemVer.isNewer('1.0.0', '1.0.0-alpha'), isFalse);
    });

    test('handles comparison operators and equality', () {
      final v020 = SemVer.parse('0.2.0');
      final v021 = SemVer.parse('v0.2.1');
      final v020b = SemVer.parse('v0.2.0');

      expect(v021 > v020, isTrue);
      expect(v020 < v021, isTrue);
      expect(v020 <= v020b, isTrue);
      expect(v021 >= v020, isTrue);
      expect(v020 == v020b, isTrue);
      expect(v020.hashCode, v020b.hashCode);
      expect(v020.toString(), '0.2.0');
    });
  });

  group('Release Model Parsing', () {
    const mockJson = '''
    {
      "tag_name": "v0.2.1",
      "name": "Sharkord Plus v0.2.1",
      "body": "## Changes\\n* In-app update engine\\n* UI improvements",
      "published_at": "2026-11-17T12:00:00Z",
      "html_url": "https://github.com/HomelessAvatar/sharkord_plus/releases/tag/v0.2.1",
      "assets": [
        {
          "name": "sharkord-mobile-android.apk",
          "size": 42500000,
          "browser_download_url": "https://github.com/HomelessAvatar/sharkord_plus/releases/download/v0.2.1/sharkord-mobile-android.apk"
        },
        {
          "name": "sharkord-desktop-windows.msi",
          "size": 65000000,
          "browser_download_url": "https://github.com/HomelessAvatar/sharkord_plus/releases/download/v0.2.1/sharkord-desktop-windows.msi"
        }
      ]
    }
    ''';

    test('parses ReleaseInfo and ReleaseAsset from GitHub JSON', () {
      final data = jsonDecode(mockJson) as Map<String, dynamic>;
      final release = ReleaseInfo.fromJson(data);

      expect(release.tagName, 'v0.2.1');
      expect(release.name, 'Sharkord Plus v0.2.1');
      expect(release.body, contains('In-app update engine'));
      expect(release.htmlUrl, contains('releases/tag/v0.2.1'));
      expect(release.assets.length, 2);

      final apk = release.apkAsset;
      expect(apk, isNotNull);
      expect(apk!.name, 'sharkord-mobile-android.apk');
      expect(apk.isApk, isTrue);
      expect(apk.size, 42500000);
      expect(apk.downloadUrl, contains('sharkord-mobile-android.apk'));

      final jsonOutput = release.toJson();
      expect(jsonOutput['tag_name'], 'v0.2.1');
      expect((jsonOutput['assets'] as List).length, 2);
    });

    test('returns null apkAsset if no apk file is present', () {
      final release = ReleaseInfo(
        tagName: 'v1.0.0',
        name: 'V1',
        body: '',
        publishedAt: '',
        htmlUrl: 'https://example.com',
        assets: const [
          ReleaseAsset(name: 'sharkord.tar.gz', size: 100, downloadUrl: 'https://example.com/file'),
        ],
      );

      expect(release.apkAsset, isNull);
    });
  });

  group('UpdateService', () {
    const mockSuccessResponse = '''
    {
      "tag_name": "v0.3.0",
      "name": "Sharkord Plus v0.3.0",
      "body": "Major update",
      "published_at": "2026-11-17T12:00:00Z",
      "html_url": "https://github.com/HomelessAvatar/sharkord_plus/releases/tag/v0.3.0",
      "assets": [
        {
          "name": "sharkord-mobile.apk",
          "size": 35000000,
          "browser_download_url": "https://example.com/sharkord-mobile.apk"
        }
      ]
    }
    ''';

    test('detects available update on Android with apk asset', () async {
      final mockClient = MockClient((request) async {
        expect(request.url.path, '/repos/HomelessAvatar/sharkord_plus/releases/latest');
        return http.Response(mockSuccessResponse, 200, headers: {'content-type': 'application/json'});
      });

      final service = UpdateService(client: mockClient);
      final result = await service.checkForUpdate(
        currentVersionOverride: '0.2.0',
        isAndroidOverride: true,
      );

      expect(result.hasUpdate, isTrue);
      expect(result.currentVersion, '0.2.0');
      expect(result.latestRelease?.tagName, 'v0.3.0');
      expect(result.targetAsset?.name, 'sharkord-mobile.apk');
    });

    test('detects available update on iOS without targetAsset but with htmlUrl', () async {
      final mockClient = MockClient((request) async {
        return http.Response(mockSuccessResponse, 200, headers: {'content-type': 'application/json'});
      });

      final service = UpdateService(client: mockClient);
      final result = await service.checkForUpdate(
        currentVersionOverride: '0.2.0',
        isAndroidOverride: false,
      );

      expect(result.hasUpdate, isTrue);
      expect(result.currentVersion, '0.2.0');
      expect(result.latestRelease?.htmlUrl, contains('releases/tag/v0.3.0'));
      expect(result.targetAsset, isNull);
    });

    test('returns hasUpdate false when on latest version', () async {
      final mockClient = MockClient((request) async {
        return http.Response(mockSuccessResponse, 200, headers: {'content-type': 'application/json'});
      });

      final service = UpdateService(client: mockClient);
      final result = await service.checkForUpdate(
        currentVersionOverride: '0.3.0',
        isAndroidOverride: true,
      );

      expect(result.hasUpdate, isFalse);
    });

    test('handles API HTTP error gracefully', () async {
      final mockClient = MockClient((request) async {
        return http.Response('Not Found', 404);
      });

      final service = UpdateService(client: mockClient);
      final result = await service.checkForUpdate(
        currentVersionOverride: '0.2.0',
      );

      expect(result.hasUpdate, isFalse);
    });
  });

  group('AndroidUpdater & IosUpdater', () {
    test('AndroidUpdater streams download with progress and invokes open file launcher', () async {
      const testFilePath = 'dummy_test_apk.apk';
      addTearDown(() {
        final f = File(testFilePath);
        if (f.existsSync()) {
          f.deleteSync();
        }
      });

      final mockBytes = Uint8List.fromList(List.generate(1024 * 10, (i) => i % 256));
      final mockClient = MockClient((request) async {
        return http.Response.bytes(
          mockBytes,
          200,
          headers: {'content-length': mockBytes.length.toString()},
        );
      });

      final updater = const AndroidUpdater();
      final progressValues = <double>[];
      String? launchedFile;

      final savedPath = await updater.downloadAndInstallApk(
        'https://example.com/app.apk',
        client: mockClient,
        targetPathOverride: testFilePath,
        onProgress: (progress, received, total) {
          progressValues.add(progress);
        },
        openFileLauncher: (filePath) async {
          launchedFile = filePath;
        },
      );

      expect(savedPath, testFilePath);
      expect(launchedFile, testFilePath);
      expect(progressValues, isNotEmpty);
      expect(progressValues.last, 1.0);
      expect(File(testFilePath).existsSync(), isTrue);
    });

    test('AndroidUpdater throws exception on non-200 download response', () async {
      final mockClient = MockClient((request) async {
        return http.Response('Error', 500);
      });

      const updater = AndroidUpdater();
      expect(
        () => updater.downloadAndInstallApk(
          'https://example.com/error.apk',
          client: mockClient,
        ),
        throwsA(isA<HttpException>()),
      );
    });

    test('IosUpdater invokes update URL launcher with externalApplication mode', () async {
      const updater = IosUpdater();
      Uri? openedUri;

      final success = await updater.openUpdatePage(
        'https://github.com/HomelessAvatar/sharkord_plus/releases/tag/v0.3.0',
        launcher: (uri, {mode}) async {
          openedUri = uri;
          return true;
        },
      );

      expect(success, isTrue);
      expect(openedUri?.toString(), contains('releases/tag/v0.3.0'));
    });
  });

  group('UpdateDialog Widget', () {
    final updateResult = UpdateCheckResult(
      hasUpdate: true,
      currentVersion: '0.2.0',
      latestRelease: ReleaseInfo(
        tagName: 'v0.3.0',
        name: 'Sharkord Plus v0.3.0',
        body: 'New voice features and performance upgrades.',
        publishedAt: '2026-11-17T12:00:00Z',
        htmlUrl: 'https://github.com/HomelessAvatar/sharkord_plus/releases/tag/v0.3.0',
        assets: const [
          ReleaseAsset(
            name: 'sharkord-mobile.apk',
            size: 30000000,
            downloadUrl: 'https://example.com/app.apk',
          ),
        ],
      ),
      targetAsset: const ReleaseAsset(
        name: 'sharkord-mobile.apk',
        size: 30000000,
        downloadUrl: 'https://example.com/app.apk',
      ),
    );

    testWidgets('renders update dialog with version tag, changelog, and buttons', (tester) async {
      bool dismissed = false;

      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData(
            scaffoldBackgroundColor: SharkordColors.background,
            dialogTheme: const DialogThemeData(
              backgroundColor: SharkordColors.card,
            ),
          ),
          home: Scaffold(
            body: UpdateDialog(
              updateResult: updateResult,
              onDismiss: () => dismissed = true,
            ),
          ),
        ),
      );

      expect(find.text('Update Available'), findsOneWidget);
      expect(find.text('v0.3.0'), findsOneWidget);
      expect(find.textContaining('Current: 0.2.0'), findsOneWidget);
      expect(find.textContaining('New voice features'), findsOneWidget);
      expect(find.text('Download & Install'), findsOneWidget);
      expect(find.text('Later'), findsOneWidget);

      await tester.tap(find.text('Later'));
      expect(dismissed, isTrue);
    });

    testWidgets('triggers onInstallCustom when primary action button tapped', (tester) async {
      bool installed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: UpdateDialog(
              updateResult: updateResult,
              onInstallCustom: () async {
                installed = true;
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Download & Install'));
      expect(installed, isTrue);
    });

    testWidgets('renders iOS update dialog with Open Update Page button when targetAsset is null', (tester) async {
      final iosResult = UpdateCheckResult(
        hasUpdate: true,
        currentVersion: '0.2.0',
        latestRelease: updateResult.latestRelease,
        targetAsset: null,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: UpdateDialog(
              updateResult: iosResult,
            ),
          ),
        ),
      );

      expect(find.text('Open Update Page'), findsOneWidget);
      expect(find.text('Later'), findsOneWidget);
    });
  });
}
