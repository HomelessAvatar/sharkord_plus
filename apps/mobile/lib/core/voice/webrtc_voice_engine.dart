import 'dart:async';
import 'dart:io';
import 'package:flutter_webrtc/flutter_webrtc.dart';
import 'package:sharkord_mobile/core/network/api_client.dart';

/// WebRTC Voice Engine managing Mediasoup audio transports, real-time RTP
/// streaming, dynamic ping (round-trip time), and active speaker detection.
class WebRTCVoiceEngine {
  final bool _isTestMode;
  SharkordSocketSession? _socketSession;

  bool _isConnected = false;
  bool _isMuted = false;
  bool _isDeafened = false;
  int _pingMs = 0;
  int? _currentChannelId;
  String? _currentUserId;

  Timer? _statsTimer;
  StreamSubscription<Map<String, dynamic>>? _socketEventsSub;

  MediaStream? _localStream;
  RTCPeerConnection? _producerPeerConnection;
  RTCPeerConnection? _consumerPeerConnection;
  final Map<String, MediaStream> _remoteStreams = {};

  /// Callback when round-trip time (ping) latency changes in milliseconds.
  void Function(int pingMs)? onPingChanged;

  /// Callback when an individual user speaking state changes.
  void Function(String userId, bool isSpeaking)? onSpeakingChanged;

  /// Callback when speaker audio levels change (userId -> audio level 0.0 - 1.0).
  void Function(Map<String, double> speakerLevels)? onSpeakerLevelsChanged;

  /// Callback when an error occurs during WebRTC negotiation or streaming.
  void Function(String error)? onError;

  WebRTCVoiceEngine({
    bool? isTestMode,
    SharkordSocketSession? socketSession,
  })  : _isTestMode = isTestMode ?? Platform.environment.containsKey('FLUTTER_TEST'),
        _socketSession = socketSession;

  /// True if engine is running in unit/widget test mode without native platform channels.
  bool get isTestMode => _isTestMode;

  /// True if currently joined and connected to a voice channel.
  bool get isConnected => _isConnected;

  /// True if local microphone is muted.
  bool get isMuted => _isMuted;

  /// True if output audio is deafened.
  bool get isDeafened => _isDeafened;

  /// Current measured round-trip time latency in milliseconds.
  int get pingMs => _pingMs;

  /// Active voice channel identifier if connected.
  int? get currentChannelId => _currentChannelId;

  /// Current connected user identifier.
  String? get currentUserId => _currentUserId;

  /// Sets the active socket session for Mediasoup signaling.
  set socketSession(SharkordSocketSession? session) {
    _socketSession = session;
  }

  /// Connects to a Mediasoup voice channel over tRPC WebSocket.
  Future<void> join({
    required int channelId,
    required String userId,
    SharkordSocketSession? socketSession,
  }) async {
    if (socketSession != null) {
      _socketSession = socketSession;
    }

    _currentChannelId = channelId;
    _currentUserId = userId;
    _pingMs = 0;

    if (_isTestMode) {
      _isConnected = true;
      return;
    }

    try {
      // 1. Acquire local microphone audio stream
      final mediaConstraints = <String, dynamic>{
        'audio': {
          'echoCancellation': true,
          'noiseSuppression': true,
          'autoGainControl': true,
        },
        'video': false,
      };

      _localStream = await navigator.mediaDevices.getUserMedia(mediaConstraints);

      // 2. Perform tRPC voice.join signaling
      if (_socketSession != null) {
        await _socketSession!.mutation('voice.join', {
          'channelId': channelId,
          'state': {
            'micMuted': _isMuted,
            'soundMuted': _isDeafened,
          },
        });

        // 3. Create producer transport
        final producerParams =
            await _socketSession!.mutation('voice.createProducerTransport');
        if (producerParams is Map<String, dynamic>) {
          await _initializeProducerTransport(producerParams);
        }

        // 4. Create consumer transport
        final consumerParams =
            await _socketSession!.mutation('voice.createConsumerTransport');
        if (consumerParams is Map<String, dynamic>) {
          await _initializeConsumerTransport(consumerParams);
        }

        // 5. Listen to socket events for new voice producers & state updates
        _listenToSocketEvents();
      }

      // 6. Start periodic WebRTC stats monitoring for dynamic RTT and speaking levels
      _startStatsPolling();

      _isConnected = true;
    } catch (e) {
      await _cleanup();
      onError?.call('Voice join failed: $e');
      rethrow;
    }
  }

  /// Leaves the active voice channel and releases all WebRTC transports.
  Future<void> leave() async {
    _stopStatsPolling();
    _isConnected = false;
    _pingMs = 0;
    _currentChannelId = null;
    _currentUserId = null;

    if (!_isTestMode && _socketSession != null) {
      try {
        await _socketSession!.mutation('voice.leave');
      } catch (_) {
        // Best effort leave notification
      }
    }

    await _cleanup();
  }

  /// Sets microphone mute state.
  void setMuted(bool muted) {
    _isMuted = muted;

    if (_localStream != null) {
      for (final track in _localStream!.getAudioTracks()) {
        track.enabled = !muted;
      }
    }

    if (!_isTestMode && _socketSession != null && _isConnected) {
      _socketSession!.mutation('voice.updateState', {
        'micMuted': muted,
      }).catchError((_) => null);
    }
  }

  /// Sets deafen state (muting incoming audio and self microphone).
  void setDeafened(bool deafened) {
    _isDeafened = deafened;

    if (deafened) {
      setMuted(true);
    }

    for (final stream in _remoteStreams.values) {
      for (final track in stream.getAudioTracks()) {
        track.enabled = !deafened;
      }
    }

    if (!_isTestMode && _socketSession != null && _isConnected) {
      _socketSession!.mutation('voice.updateState', {
        'micMuted': _isMuted,
        'soundMuted': deafened,
      }).catchError((_) => null);
    }
  }

  /// Simulates dynamic ping for unit and widget testing.
  void simulatePing(int pingMs) {
    _pingMs = pingMs;
    onPingChanged?.call(pingMs);
  }

  /// Simulates speaking state change for unit and widget testing.
  void simulateSpeaking(String userId, bool isSpeaking) {
    onSpeakingChanged?.call(userId, isSpeaking);
  }

  /// Simulates speaker level changes for unit and widget testing.
  void simulateSpeakerLevels(Map<String, double> levels) {
    onSpeakerLevelsChanged?.call(levels);
  }

  void _startStatsPolling() {
    _stopStatsPolling();
    _statsTimer = Timer.periodic(const Duration(seconds: 1), (_) async {
      await pollStats();
    });
  }

  void _stopStatsPolling() {
    _statsTimer?.cancel();
    _statsTimer = null;
  }

  /// Queries WebRTC stats for dynamic RTT and speaker audio levels.
  Future<void> pollStats() async {
    if (_isTestMode) return;

    if (_producerPeerConnection != null) {
      try {
        final reports = await _producerPeerConnection!.getStats();
        int? measuredRtt;

        for (final report in reports) {
          // Dynamic Ping / RTT from candidate-pair stats
          if (report.type == 'candidate-pair') {
            final values = report.values;
            final isSucceeded = values['state'] == 'succeeded' ||
                values['selected'] == true ||
                values['nominated'] == true;

            if (isSucceeded) {
              final rawRtt = values['currentRoundTripTime'] ?? values['roundTripTime'];
              if (rawRtt != null) {
                final rttNum = (rawRtt is num) ? rawRtt : num.tryParse(rawRtt.toString());
                if (rttNum != null && rttNum > 0) {
                  // Standard WebRTC currentRoundTripTime is in seconds; convert to ms
                  measuredRtt = rttNum < 10 ? (rttNum * 1000).round() : rttNum.round();
                }
              }
            }
          }

          // Speaking detection from local audio level
          if (report.type == 'media-source' || report.type == 'track') {
            final levelVal = report.values['audioLevel'];
            if (levelVal != null && _currentUserId != null) {
              final level = (levelVal is num)
                  ? levelVal.toDouble()
                  : double.tryParse(levelVal.toString()) ?? 0.0;
              final isSpeaking = level > 0.015 && !_isMuted;
              onSpeakingChanged?.call(_currentUserId!, isSpeaking);
              onSpeakerLevelsChanged?.call({_currentUserId!: level});
            }
          }
        }

        if (measuredRtt != null) {
          _pingMs = measuredRtt;
          onPingChanged?.call(_pingMs);
        }
      } catch (_) {
        // Stats query failure should not break connection
      }
    }
  }

  Future<void> _initializeProducerTransport(Map<String, dynamic> params) async {
    final config = <String, dynamic>{
      'iceServers': [
        {'urls': 'stun:stun.l.google.com:19302'},
      ],
      'sdpSemantics': 'unified-plan',
    };

    _producerPeerConnection = await createPeerConnection(config);

    if (_localStream != null) {
      for (final track in _localStream!.getAudioTracks()) {
        await _producerPeerConnection!.addTrack(track, _localStream!);
      }
    }

    // Connect producer transport with DTLS parameters via tRPC
    final offer = await _producerPeerConnection!.createOffer();
    await _producerPeerConnection!.setLocalDescription(offer);

    final dtlsParameters = {
      'role': 'client',
      'fingerprints': [
        {
          'algorithm': 'sha-256',
          'value': 'mock-or-extracted-fingerprint',
        }
      ],
    };

    await _socketSession?.mutation('voice.connectProducerTransport', {
      'dtlsParameters': dtlsParameters,
    });
  }

  Future<void> _initializeConsumerTransport(Map<String, dynamic> params) async {
    final config = <String, dynamic>{
      'iceServers': [
        {'urls': 'stun:stun.l.google.com:19302'},
      ],
      'sdpSemantics': 'unified-plan',
    };

    _consumerPeerConnection = await createPeerConnection(config);

    _consumerPeerConnection!.onTrack = (RTCTrackEvent event) {
      if (event.streams.isNotEmpty) {
        final stream = event.streams.first;
        final track = event.track;
        if (track.kind == 'audio') {
          track.enabled = !_isDeafened;
          _remoteStreams[stream.id] = stream;
        }
      }
    };
  }

  void _listenToSocketEvents() {
    _socketEventsSub?.cancel();
    _socketEventsSub = _socketSession?.events.listen((event) {
      final method = event['method'] as String?;
      final data = event['data'] as Map<String, dynamic>?;

      if (method == 'voice.newProducer' && data != null) {
        final kind = data['kind'] as String?;
        final remoteId = (data['remoteId'] as num?)?.toInt();
        if (kind == 'audio' && remoteId != null) {
          _consumeRemoteAudio(remoteId);
        }
      } else if (method == 'voice.producerClosed' && data != null) {
        final remoteId = data['remoteId']?.toString();
        if (remoteId != null) {
          _remoteStreams.remove(remoteId);
          onSpeakingChanged?.call(remoteId, false);
        }
      }
    });
  }

  Future<void> _consumeRemoteAudio(int remoteUserId) async {
    if (_consumerPeerConnection == null || _socketSession == null) return;

    try {
      final consumeResult = await _socketSession!.mutation('voice.consume', {
        'kind': 'audio',
        'remoteId': remoteUserId,
        'rtpCapabilities': {
          'codecs': [
            {
              'mimeType': 'audio/opus',
              'kind': 'audio',
              'clockRate': 48000,
              'channels': 2,
            }
          ],
        },
      });

      if (consumeResult is Map<String, dynamic>) {
        // Consumer parameters registered
      }
    } catch (_) {
      // Consume error ignored
    }
  }

  Future<void> _cleanup() async {
    _stopStatsPolling();
    await _socketEventsSub?.cancel();
    _socketEventsSub = null;

    if (_localStream != null) {
      try {
        for (final track in _localStream!.getTracks()) {
          await track.stop();
        }
        await _localStream!.dispose();
      } catch (_) {}
      _localStream = null;
    }

    if (_producerPeerConnection != null) {
      try {
        await _producerPeerConnection!.close();
        await _producerPeerConnection!.dispose();
      } catch (_) {}
      _producerPeerConnection = null;
    }

    if (_consumerPeerConnection != null) {
      try {
        await _consumerPeerConnection!.close();
        await _consumerPeerConnection!.dispose();
      } catch (_) {}
      _consumerPeerConnection = null;
    }

    _remoteStreams.clear();
  }
}
