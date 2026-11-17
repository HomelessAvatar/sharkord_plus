import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:sharkord_mobile/core/network/api_client.dart';

void main() {
  group('SharkordApiClient URL & Parsing Tests', () {
    test('normalizeServerUrl handles domains, protocols, and trailing slashes', () {
      expect(normalizeServerUrl('lobi.genclink.tr'), 'https://lobi.genclink.tr');
      expect(normalizeServerUrl('lobi.genclink.tr/'), 'https://lobi.genclink.tr');
      expect(normalizeServerUrl('https://lobi.genclink.tr///'), 'https://lobi.genclink.tr');
      expect(normalizeServerUrl('http://192.168.1.50:3000/'), 'http://192.168.1.50:3000');
      expect(normalizeServerUrl('localhost:3000'), 'http://localhost:3000');
      expect(normalizeServerUrl('10.0.2.2:3000'), 'http://10.0.2.2:3000');
      expect(normalizeServerUrl(''), '');
    });

    test('toWsUrl correctly converts http/https to ws/wss', () {
      expect(toWsUrl('https://lobi.genclink.tr'), 'wss://lobi.genclink.tr');
      expect(toWsUrl('http://localhost:3000'), 'ws://localhost:3000');
      expect(toWsUrl('lobi.genclink.tr'), 'wss://lobi.genclink.tr');
    });

    test('fetchServerInfo parses valid server metadata', () async {
      final mockClient = MockClient((request) async {
        if (request.url.path == '/info') {
          return http.Response(
            jsonEncode({
              'serverId': 'srv_123',
              'name': 'Genclik Linki Server',
              'description': 'Official Sharkord Community',
              'logo': 'https://example.com/logo.png',
              'allowNewUsers': true,
            }),
            200,
            headers: {'content-type': 'application/json'},
          );
        }
        return http.Response('Not Found', 404);
      });

      final client = SharkordApiClient(httpClient: mockClient);
      final info = await client.fetchServerInfo('https://lobi.genclink.tr');

      expect(info.serverId, 'srv_123');
      expect(info.name, 'Genclik Linki Server');
      expect(info.description, 'Official Sharkord Community');
      expect(info.logo, 'https://example.com/logo.png');
      expect(info.allowNewUsers, true);
    });

    test('login succeeds and extracts token on HTTP 200', () async {
      final mockClient = MockClient((request) async {
        if (request.url.path == '/login') {
          final body = jsonDecode(request.body) as Map<String, dynamic>;
          if (body['identity'] == 'admin' && body['password'] == 'secret') {
            return http.Response(
              jsonEncode({
                'success': true,
                'token': 'jwt_valid_token_xyz',
              }),
              200,
              headers: {'content-type': 'application/json'},
            );
          }
          return http.Response(
            jsonEncode({'message': 'Invalid credentials'}),
            400,
            headers: {'content-type': 'application/json'},
          );
        }
        return http.Response('Not Found', 404);
      });

      final client = SharkordApiClient(httpClient: mockClient);
      final token = await client.login(
        serverUrl: 'https://lobi.genclink.tr',
        identity: 'admin',
        password: 'secret',
      );

      expect(token, 'jwt_valid_token_xyz');

      expect(
        () => client.login(
          serverUrl: 'https://lobi.genclink.tr',
          identity: 'admin',
          password: 'wrong',
        ),
        throwsA(isA<Exception>()),
      );
    });
  });
}
