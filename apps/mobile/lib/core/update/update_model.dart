import 'package:flutter/foundation.dart';

@immutable
class SemVer implements Comparable<SemVer> {
  final int major;
  final int minor;
  final int patch;
  final String? preRelease;

  const SemVer({
    required this.major,
    required this.minor,
    required this.patch,
    this.preRelease,
  });

  factory SemVer.parse(String version) {
    var clean = version.trim();
    final vIndex = clean.toLowerCase().indexOf('v');
    if (vIndex >= 0) {
      clean = clean.substring(vIndex + 1);
    }
    if (clean.contains('+')) {
      clean = clean.split('+').first;
    }

    String? pre;
    if (clean.contains('-')) {
      final parts = clean.split('-');
      clean = parts.first;
      pre = parts.sublist(1).join('-');
    }

    final segments = clean.split('.');
    final major = segments.isNotEmpty ? int.tryParse(segments[0]) ?? 0 : 0;
    final minor = segments.length > 1 ? int.tryParse(segments[1]) ?? 0 : 0;
    final patch = segments.length > 2 ? int.tryParse(segments[2]) ?? 0 : 0;

    return SemVer(
      major: major,
      minor: minor,
      patch: patch,
      preRelease: pre,
    );
  }

  static bool isNewer(String current, String target) {
    try {
      final cur = SemVer.parse(current);
      final tgt = SemVer.parse(target);
      return tgt.compareTo(cur) > 0;
    } catch (_) {
      return false;
    }
  }

  @override
  int compareTo(SemVer other) {
    if (major != other.major) return major.compareTo(other.major);
    if (minor != other.minor) return minor.compareTo(other.minor);
    if (patch != other.patch) return patch.compareTo(other.patch);

    if (preRelease == null && other.preRelease != null) return 1;
    if (preRelease != null && other.preRelease == null) return -1;
    if (preRelease != null && other.preRelease != null) {
      return preRelease!.compareTo(other.preRelease!);
    }
    return 0;
  }

  bool operator >(SemVer other) => compareTo(other) > 0;
  bool operator <(SemVer other) => compareTo(other) < 0;
  bool operator >=(SemVer other) => compareTo(other) >= 0;
  bool operator <=(SemVer other) => compareTo(other) <= 0;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SemVer &&
          runtimeType == other.runtimeType &&
          major == other.major &&
          minor == other.minor &&
          patch == other.patch &&
          preRelease == other.preRelease;

  @override
  int get hashCode => Object.hash(major, minor, patch, preRelease);

  @override
  String toString() =>
      '$major.$minor.$patch${preRelease != null ? '-$preRelease' : ''}';
}

@immutable
class ReleaseAsset {
  final String name;
  final int size;
  final String downloadUrl;

  const ReleaseAsset({
    required this.name,
    required this.size,
    required this.downloadUrl,
  });

  bool get isApk => name.toLowerCase().endsWith('.apk');

  factory ReleaseAsset.fromJson(Map<String, dynamic> json) {
    return ReleaseAsset(
      name: json['name'] as String? ?? '',
      size: (json['size'] as num?)?.toInt() ?? 0,
      downloadUrl: json['browser_download_url'] as String? ?? '',
    );
  }

  Map<String, dynamic> toJson() => {
    'name': name,
    'size': size,
    'browser_download_url': downloadUrl,
  };
}

@immutable
class ReleaseInfo {
  final String tagName;
  final String name;
  final String body;
  final String publishedAt;
  final String htmlUrl;
  final List<ReleaseAsset> assets;

  const ReleaseInfo({
    required this.tagName,
    required this.name,
    required this.body,
    required this.publishedAt,
    required this.htmlUrl,
    required this.assets,
  });

  ReleaseAsset? get apkAsset {
    for (final asset in assets) {
      if (asset.isApk) return asset;
    }
    return null;
  }

  factory ReleaseInfo.fromJson(Map<String, dynamic> json) {
    final rawAssets = json['assets'] as List<dynamic>? ?? [];
    final assets = rawAssets
        .whereType<Map<String, dynamic>>()
        .map(ReleaseAsset.fromJson)
        .toList();

    return ReleaseInfo(
      tagName: json['tag_name'] as String? ?? '',
      name: json['name'] as String? ?? '',
      body: json['body'] as String? ?? '',
      publishedAt: json['published_at'] as String? ?? '',
      htmlUrl: json['html_url'] as String? ?? '',
      assets: assets,
    );
  }

  Map<String, dynamic> toJson() => {
    'tag_name': tagName,
    'name': name,
    'body': body,
    'published_at': publishedAt,
    'html_url': htmlUrl,
    'assets': assets.map((a) => a.toJson()).toList(),
  };
}

@immutable
class UpdateCheckResult {
  final bool hasUpdate;
  final String currentVersion;
  final ReleaseInfo? latestRelease;
  final ReleaseAsset? targetAsset;

  const UpdateCheckResult({
    required this.hasUpdate,
    required this.currentVersion,
    this.latestRelease,
    this.targetAsset,
  });
}
