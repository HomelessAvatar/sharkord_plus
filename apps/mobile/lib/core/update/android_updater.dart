import 'dart:io';
import 'package:http/http.dart' as http;
import 'package:open_filex/open_filex.dart';
import 'package:path_provider/path_provider.dart';

class AndroidUpdater {
  const AndroidUpdater();

  Future<String> downloadAndInstallApk(
    String downloadUrl, {
    http.Client? client,
    String? targetPathOverride,
    void Function(double progress, int received, int total)? onProgress,
    Future<dynamic> Function(String filePath)? openFileLauncher,
  }) async {
    final effectiveClient = client ?? http.Client();
    final shouldCloseClient = client == null;

    try {
      final request = http.Request('GET', Uri.parse(downloadUrl));
      final response = await effectiveClient.send(request);

      if (response.statusCode != 200) {
        throw HttpException('Failed to download update: HTTP ${response.statusCode}');
      }

      final total = response.contentLength ?? -1;
      int received = 0;

      final String filePath;
      if (targetPathOverride != null) {
        filePath = targetPathOverride;
      } else {
        final dir = await getTemporaryDirectory();
        filePath = '${dir.path}/sharkord_update.apk';
      }

      final file = File(filePath);
      final sink = file.openWrite();

      await for (final chunk in response.stream) {
        sink.add(chunk);
        received += chunk.length;
        if (onProgress != null) {
          final progress = total > 0 ? (received / total).clamp(0.0, 1.0) : 0.0;
          onProgress(progress, received, total);
        }
      }

      await sink.flush();
      await sink.close();

      if (openFileLauncher != null) {
        await openFileLauncher(filePath);
      } else {
        await OpenFilex.open(filePath);
      }

      return filePath;
    } finally {
      if (shouldCloseClient) {
        effectiveClient.close();
      }
    }
  }
}
