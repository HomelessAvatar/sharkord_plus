import 'package:url_launcher/url_launcher.dart';

class IosUpdater {
  const IosUpdater();

  Future<bool> openUpdatePage(
    String url, {
    Future<bool> Function(Uri uri, {LaunchMode? mode})? launcher,
  }) async {
    final uri = Uri.parse(url);
    if (launcher != null) {
      return launcher(uri, mode: LaunchMode.externalApplication);
    }
    return launchUrl(uri, mode: LaunchMode.externalApplication);
  }
}
