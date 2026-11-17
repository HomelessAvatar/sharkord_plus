import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import 'package:sharkord_mobile/models/channel.dart';
import 'package:sharkord_mobile/models/message.dart';
import 'package:sharkord_mobile/models/user.dart';

/// Server information returned by GET /info
class ServerInfo {
  final String serverId;
  final String name;
  final String? description;
  final String? logo;
  final bool allowNewUsers;

  const ServerInfo({
    required this.serverId,
    required this.name,
    this.description,
    this.logo,
    this.allowNewUsers = true,
  });

  factory ServerInfo.fromJson(Map<String, dynamic> json) {
    return ServerInfo(
      serverId: json['serverId']?.toString() ?? '',
      name: json['name'] as String? ?? 'Sharkord Server',
      description: json['description'] as String?,
      logo: json['logo'] as String?,
      allowNewUsers: json['allowNewUsers'] as bool? ?? true,
    );
  }
}

/// Initial server state returned by others.joinServer
class JoinServerResult {
  final String serverId;
  final String serverName;
  final int ownUserId;
  final List<Channel> channels;
  final List<User> users;
  final Map<int, String> categoryNames;

  const JoinServerResult({
    required this.serverId,
    required this.serverName,
    required this.ownUserId,
    required this.channels,
    required this.users,
    this.categoryNames = const {},
  });
}

/// Normalizes user-entered server URL into a valid HTTP/HTTPS URL.
String normalizeServerUrl(String input) {
  var trimmed = input.trim();
  if (trimmed.isEmpty) return '';

  if (!trimmed.startsWith('http://') && !trimmed.startsWith('https://')) {
    // Default to https, except for localhost / 10.0.2.2 / local dev IPs
    if (trimmed.startsWith('localhost') ||
        trimmed.startsWith('127.0.0.1') ||
        trimmed.startsWith('10.0.2.2') ||
        trimmed.contains(':3000')) {
      trimmed = 'http://$trimmed';
    } else {
      trimmed = 'https://$trimmed';
    }
  }

  while (trimmed.endsWith('/')) {
    trimmed = trimmed.substring(0, trimmed.length - 1);
  }

  return trimmed;
}

/// Derives WebSocket URL (ws:// or wss://) from normalized server URL.
String toWsUrl(String serverUrl) {
  final normalized = normalizeServerUrl(serverUrl);
  if (normalized.startsWith('https://')) {
    return 'wss://${normalized.substring('https://'.length)}';
  } else if (normalized.startsWith('http://')) {
    return 'ws://${normalized.substring('http://'.length)}';
  }
  return 'wss://$normalized';
}

/// HTTP and tRPC WebSocket client for Sharkord.
class SharkordApiClient {
  final http.Client _httpClient;

  SharkordApiClient({http.Client? httpClient})
      : _httpClient = httpClient ?? http.Client();

  /// Fetches public server metadata via GET /info
  Future<ServerInfo> fetchServerInfo(String serverUrl) async {
    final cleanUrl = normalizeServerUrl(serverUrl);
    final uri = Uri.parse('$cleanUrl/info');
    final response = await _httpClient.get(uri).timeout(
          const Duration(seconds: 10),
          onTimeout: () => throw TimeoutException('Connection to $cleanUrl timed out'),
        );

    if (response.statusCode != 200) {
      throw HttpException(
        'Failed to fetch server info: HTTP ${response.statusCode}',
        uri: uri,
      );
    }

    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return ServerInfo.fromJson(data);
  }

  /// Authenticates user credentials via POST /login and returns JWT token
  Future<String> login({
    required String serverUrl,
    required String identity,
    required String password,
  }) async {
    final cleanUrl = normalizeServerUrl(serverUrl);
    final uri = Uri.parse('$cleanUrl/login');

    final response = await _httpClient
        .post(
          uri,
          headers: {'Content-Type': 'application/json'},
          body: jsonEncode({
            'identity': identity.trim().toLowerCase(),
            'password': password,
          }),
        )
        .timeout(
          const Duration(seconds: 12),
          onTimeout: () => throw TimeoutException('Login to $cleanUrl timed out'),
        );

    final dynamic bodyData = jsonDecode(response.body);

    if (response.statusCode == 200) {
      if (bodyData is Map<String, dynamic> && bodyData['token'] != null) {
        return bodyData['token'] as String;
      }
      throw const FormatException('Server response missing token');
    }

    // Handle error messages from server
    if (bodyData is Map<String, dynamic>) {
      if (bodyData['message'] != null) {
        throw HttpException(bodyData['message'].toString(), uri: uri);
      }
      if (bodyData['error'] != null) {
        throw HttpException(bodyData['error'].toString(), uri: uri);
      }
    }

    throw HttpException('Login failed with status ${response.statusCode}', uri: uri);
  }

  /// Connects to the tRPC WebSocket endpoint and returns an active session.
  Future<SharkordSocketSession> connectSession({
    required String serverUrl,
    required String token,
  }) async {
    final wsUrl = '${toWsUrl(serverUrl)}/?connectionParams=1';
    final socket = await WebSocket.connect(wsUrl).timeout(
      const Duration(seconds: 10),
      onTimeout: () => throw TimeoutException('WebSocket connection to $wsUrl timed out'),
    );

    final session = SharkordSocketSession(socket: socket, token: token);
    await session.initialize();
    return session;
  }
}

/// Active WebSocket session communicating over tRPC v11 JSON-RPC protocol.
class SharkordSocketSession {
  final WebSocket _socket;
  final String _token;
  int _nextRequestId = 1;
  final Map<int, Completer<dynamic>> _pendingRequests = {};
  final StreamController<Map<String, dynamic>> _eventController =
      StreamController<Map<String, dynamic>>.broadcast();

  bool _isDisposed = false;

  SharkordSocketSession({
    required WebSocket socket,
    required String token,
  })  : _socket = socket,
        _token = token;

  /// Stream of real-time server events (user joins, messages, etc.)
  Stream<Map<String, dynamic>> get events => _eventController.stream;

  /// Initializes connection by sending connectionParams
  Future<void> initialize() async {
    _socket.listen(
      _handleIncomingMessage,
      onError: (err) {
        _failPending(err);
      },
      onDone: () {
        _failPending('WebSocket connection closed');
      },
    );

    // Send connection params as first frame
    _socket.add(jsonEncode({
      'method': 'connectionParams',
      'data': {'token': _token},
    }));

    // Small delay to ensure context is instantiated on server
    await Future<void>.delayed(const Duration(milliseconds: 50));
  }

  void _handleIncomingMessage(dynamic raw) {
    if (raw is! String) return;

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return;

      // Handle keep-alive ping
      if (decoded['method'] == 'ping') {
        _socket.add(jsonEncode({'method': 'pong'}));
        return;
      }

      // Handle response to pending request
      final id = decoded['id'];
      if (id is int && _pendingRequests.containsKey(id)) {
        final completer = _pendingRequests.remove(id)!;
        if (decoded['error'] != null) {
          final errMap = decoded['error'] as Map<String, dynamic>;
          final msg = errMap['message'] ?? 'tRPC Request Error';
          completer.completeError(HttpException(msg.toString()));
        } else if (decoded['result'] != null) {
          final result = decoded['result'] as Map<String, dynamic>;
          completer.complete(result['data']);
        } else {
          completer.complete(null);
        }
        return;
      }

      // Handle subscription / broadcast events
      _eventController.add(decoded);
    } catch (_) {
      // Ignore malformed frames
    }
  }

  void _failPending(dynamic error) {
    final err = error is Exception ? error : Exception(error.toString());
    for (final completer in _pendingRequests.values) {
      if (!completer.isCompleted) {
        completer.completeError(err);
      }
    }
    _pendingRequests.clear();
  }

  /// Sends a tRPC query
  Future<dynamic> query(String path, [dynamic input]) {
    return _sendRequest('query', path, input);
  }

  /// Sends a tRPC mutation
  Future<dynamic> mutation(String path, [dynamic input]) {
    return _sendRequest('mutation', path, input);
  }

  Future<dynamic> _sendRequest(String method, String path, dynamic input) {
    if (_isDisposed) {
      return Future.error(StateError('Socket session is disposed'));
    }

    final id = _nextRequestId++;
    final completer = Completer<dynamic>();
    _pendingRequests[id] = completer;

    final request = <String, dynamic>{
      'id': id,
      'method': method,
      'params': <String, dynamic>{
        'path': path,
        if (input != null) 'input': input,
      },
    };

    _socket.add(jsonEncode(request));

    return completer.future.timeout(
      const Duration(seconds: 15),
      onTimeout: () {
        _pendingRequests.remove(id);
        throw TimeoutException('Request $path timed out');
      },
    );
  }

  /// Performs server handshake
  Future<String> handshake() async {
    final response = await query('others.handshake');
    if (response is Map<String, dynamic> && response['handshakeHash'] != null) {
      return response['handshakeHash'] as String;
    }
    throw const FormatException('Invalid handshake response');
  }

  /// Joins server and parses initial state (channels, users, categories)
  Future<JoinServerResult> joinServer({
    required String handshakeHash,
    String? password,
  }) async {
    final dynamic response = await query('others.joinServer', {
      'handshakeHash': handshakeHash,
      if (password != null) 'password': password,
    });

    if (response is! Map<String, dynamic>) {
      throw const FormatException('Invalid joinServer response shape');
    }

    final serverId = response['serverId']?.toString() ?? '';
    final serverName = response['serverName'] as String? ?? 'Sharkord Server';
    final ownUserId = (response['ownUserId'] as num?)?.toInt() ?? 0;

    // Parse categories map: id -> name
    final categoryMap = <int, String>{};
    if (response['categories'] is List) {
      for (final cat in response['categories'] as List) {
        if (cat is Map<String, dynamic>) {
          final cid = (cat['id'] as num?)?.toInt();
          final cname = cat['name'] as String?;
          if (cid != null && cname != null) {
            categoryMap[cid] = cname;
          }
        }
      }
    }

    // Parse channels
    final channelsList = <Channel>[];
    if (response['channels'] is List) {
      for (final ch in response['channels'] as List) {
        if (ch is Map<String, dynamic>) {
          final catId = (ch['categoryId'] as num?)?.toInt();
          final categoryName = catId != null ? categoryMap[catId] : null;

          final channel = Channel(
            id: ch['id']?.toString() ?? '',
            name: ch['name'] as String? ?? '',
            category: categoryName ?? (ch['type'] == 'VOICE' ? 'VOICE CHANNELS' : 'TEXT CHANNELS'),
            isVoice: ch['type'] == 'VOICE' || ch['channelType'] == 'VOICE',
            participantCount: (ch['participantCount'] as num?)?.toInt() ?? 0,
          );
          channelsList.add(channel);
        }
      }
    }

    // Parse users
    final usersList = <User>[];
    if (response['users'] is List) {
      for (final u in response['users'] as List) {
        if (u is Map<String, dynamic>) {
          usersList.add(User.fromJson(u));
        }
      }
    }

    return JoinServerResult(
      serverId: serverId,
      serverName: serverName,
      ownUserId: ownUserId,
      channels: channelsList,
      users: usersList,
      categoryNames: categoryMap,
    );
  }

  /// Fetches messages for a channel
  Future<List<Message>> fetchMessages(int channelId, {int limit = 50}) async {
    final dynamic response = await query('messages.getMessages', {
      'channelId': channelId,
      'limit': limit,
    });

    if (response is Map<String, dynamic> && response['messages'] is List) {
      final list = <Message>[];
      for (final m in response['messages'] as List) {
        if (m is Map<String, dynamic>) {
          list.add(Message.fromJson(m));
        }
      }
      return list;
    }
    return const [];
  }

  /// Sends a message to a channel
  Future<Message> sendMessage(int channelId, String content) async {
    final dynamic response = await mutation('messages.sendMessage', {
      'channelId': channelId,
      'content': content,
      'files': <String>[],
    });

    if (response is Map<String, dynamic>) {
      return Message.fromJson(response);
    }
    throw const FormatException('Invalid sendMessage response');
  }

  /// Disposes session and closes socket
  Future<void> dispose() async {
    if (_isDisposed) return;
    _isDisposed = true;
    _failPending('Session disposed');
    await _eventController.close();
    await _socket.close();
  }
}
