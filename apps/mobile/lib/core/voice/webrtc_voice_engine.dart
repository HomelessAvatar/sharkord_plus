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
  String? _audioProducerId;

  Timer? _statsTimer;
  StreamSubscription<Map<String, dynamic>>? _socketEventsSub;

  MediaStream? _localStream;
  RTCPeerConnection? _producerPeerConnection;
  RTCPeerConnection? _consumerPeerConnection;
  final Map<String, MediaStream> _remoteStreams = {};
  final Map<String, String> _consumerToUserId = {};
  final Map<String, bool> _previousSpeakingStates = {};

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
  })  : _isTestMode =
            isTestMode ?? Platform.environment.containsKey('FLUTTER_TEST'),
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

  /// Active audio producer ID if registered on Mediasoup SFU router.
  String? get audioProducerId => _audioProducerId;

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
    _previousSpeakingStates.clear();

    if (_isTestMode) {
      _isConnected = true;
      return;
    }

    try {
      // 1. Acquire local microphone audio stream with DSP constraints
      final mediaConstraints = <String, dynamic>{
        'audio': {
          'echoCancellation': true,
          'noiseSuppression': true,
          'autoGainControl': true,
        },
        'video': false,
      };

      _localStream =
          await navigator.mediaDevices.getUserMedia(mediaConstraints);

      // 2. Perform tRPC voice.join signaling
      if (_socketSession != null) {
        await _socketSession!.mutation('voice.join', {
          'channelId': channelId,
          'state': {
            'micMuted': _isMuted,
            'soundMuted': _isDeafened,
          },
        });

        // 3. Create & connect producer transport, then produce audio track
        final producerParams =
            await _socketSession!.mutation('voice.createProducerTransport');
        if (producerParams is Map<String, dynamic>) {
          await _initializeProducerTransport(producerParams);
        }

        // 4. Create & connect consumer transport for incoming streams
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
    _audioProducerId = null;
    _previousSpeakingStates.clear();

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
    _previousSpeakingStates[userId] = isSpeaking;
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

    final speakerLevels = <String, double>{};

    // 1. Producer stats: candidate-pair RTT and local microphone audio level
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
              final rawRtt =
                  values['currentRoundTripTime'] ?? values['roundTripTime'];
              if (rawRtt != null) {
                final rttNum =
                    (rawRtt is num) ? rawRtt : num.tryParse(rawRtt.toString());
                if (rttNum != null && rttNum > 0) {
                  // WebRTC currentRoundTripTime is in seconds; convert to ms
                  measuredRtt =
                      rttNum < 10 ? (rttNum * 1000).round() : rttNum.round();
                }
              }
            }
          }

          // Local speaking detection from audio track / media-source
          if (report.type == 'media-source' || report.type == 'track') {
            final levelVal = report.values['audioLevel'];
            if (levelVal != null && _currentUserId != null) {
              final level = (levelVal is num)
                  ? levelVal.toDouble()
                  : double.tryParse(levelVal.toString()) ?? 0.0;
              final isSpeaking = level > 0.015 && !_isMuted;
              speakerLevels[_currentUserId!] = level;

              // Only notify if speaking state transitioned
              if (_previousSpeakingStates[_currentUserId!] != isSpeaking) {
                _previousSpeakingStates[_currentUserId!] = isSpeaking;
                onSpeakingChanged?.call(_currentUserId!, isSpeaking);
              }
            }
          }
        }

        if (measuredRtt != null && measuredRtt != _pingMs) {
          _pingMs = measuredRtt;
          onPingChanged?.call(_pingMs);
        }
      } catch (_) {
        // Ignore transient stats errors
      }
    }

    // 2. Consumer stats: incoming audio levels for remote participants
    if (_consumerPeerConnection != null) {
      try {
        final consumerReports = await _consumerPeerConnection!.getStats();
        for (final report in consumerReports) {
          if (report.type == 'inbound-rtp' || report.type == 'track') {
            final levelVal = report.values['audioLevel'];
            final trackId = report.values['trackIdentifier']?.toString() ??
                report.id.toString();

            // Match track to remote user via consumerToUserId map
            String? matchedUserId;
            for (final entry in _consumerToUserId.entries) {
              if (trackId.contains(entry.key) || entry.key.contains(trackId)) {
                matchedUserId = entry.value;
                break;
              }
            }

            if (matchedUserId != null && levelVal != null) {
              final level = (levelVal is num)
                  ? levelVal.toDouble()
                  : double.tryParse(levelVal.toString()) ?? 0.0;
              final isSpeaking = level > 0.015 && !_isDeafened;
              speakerLevels[matchedUserId] = level;

              if (_previousSpeakingStates[matchedUserId] != isSpeaking) {
                _previousSpeakingStates[matchedUserId] = isSpeaking;
                onSpeakingChanged?.call(matchedUserId, isSpeaking);
              }
            }
          }
        }
      } catch (_) {
        // Ignore transient consumer stats errors
      }
    }

    if (speakerLevels.isNotEmpty) {
      onSpeakerLevelsChanged?.call(speakerLevels);
    }
  }

  /// Extracts real DTLS parameters from local SDP session description.
  Map<String, dynamic> extractDtlsParameters(String sdp,
      {String role = 'client'}) {
    final match = RegExp(r'a=fingerprint:(\S+)\s+(\S+)').firstMatch(sdp);
    final algorithm = match?.group(1) ?? 'sha-256';
    final value = match?.group(2) ?? '';
    return {
      'role': role,
      'fingerprints': [
        {
          'algorithm': algorithm,
          'value': value,
        }
      ],
    };
  }

  /// Extracts standard Mediasoup audio RTP parameters from local SDP.
  Map<String, dynamic> extractRtpParameters(String sdp) {
    final ssrcMatch = RegExp(r'a=ssrc:(\d+)').firstMatch(sdp);
    final ssrc = ssrcMatch != null ? int.tryParse(ssrcMatch.group(1)!) : 10001;
    final cnameMatch = RegExp(r'a=ssrc:\d+ cname:(\S+)').firstMatch(sdp);
    final cname = cnameMatch?.group(1) ?? 'sharkord-mobile';
    final midMatch = RegExp(r'a=mid:(\S+)').firstMatch(sdp);
    final mid = midMatch?.group(1) ?? '0';

    return {
      'mid': mid,
      'codecs': [
        {
          'mimeType': 'audio/opus',
          'payloadType': 111,
          'clockRate': 48000,
          'channels': 2,
          'parameters': {
            'useinbandfec': 1,
            'usedtx': 1,
            'stereo': 1,
          },
          'rtcpFeedback': <dynamic>[],
        }
      ],
      'headerExtensions': [
        {
          'uri': 'urn:ietf:params:rtp-hdrext:sdes:mid',
          'id': 1,
        },
        {
          'uri': 'urn:ietf:params:rtp-hdrext:ssrc-audio-level',
          'id': 10,
        }
      ],
      'encodings': [
        {
          'ssrc': ssrc,
          'dtx': true,
        }
      ],
      'rtcp': {
        'cname': cname,
        'reducedSize': true,
      },
    };
  }

  /// Synthesizes remote SDP Answer from Mediasoup transport parameters for peer connection.
  String buildRemoteAnswerSdp({
    required String localSdp,
    required Map<String, dynamic> transportParams,
  }) {
    final iceParams =
        transportParams['iceParameters'] as Map<String, dynamic>? ?? {};
    final iceCandidates =
        (transportParams['iceCandidates'] as List<dynamic>?) ?? [];
    final dtlsParams =
        transportParams['dtlsParameters'] as Map<String, dynamic>? ?? {};
    final fingerprints = (dtlsParams['fingerprints'] as List<dynamic>?) ?? [];

    final ufrag = iceParams['usernameFragment']?.toString() ?? 'server-ufrag';
    final pwd = iceParams['password']?.toString() ?? 'server-pwd';

    final fp = fingerprints.isNotEmpty && fingerprints.first is Map
        ? fingerprints.first as Map<String, dynamic>
        : {
            'algorithm': 'sha-256',
            'value':
                '00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00'
          };
    final fpAlgo = fp['algorithm'] ?? 'sha-256';
    final fpVal = fp['value'] ?? '';

    final midMatch = RegExp(r'a=mid:(\S+)').firstMatch(localSdp);
    final mid = midMatch?.group(1) ?? '0';

    final lines = <String>[
      'v=0',
      'o=- 100000 2 IN IP4 127.0.0.1',
      's=-',
      't=0 0',
      'a=group:BUNDLE $mid',
      'm=audio 9 UDP/TLS/RTP/SAVPF 111',
      'c=IN IP4 0.0.0.0',
      'a=mid:$mid',
      'a=ice-ufrag:$ufrag',
      'a=ice-pwd:$pwd',
      if (iceParams['iceLite'] == true) 'a=ice-lite',
      'a=fingerprint:$fpAlgo $fpVal',
      'a=setup:passive',
      'a=rtpmap:111 opus/48000/2',
      'a=fmtp:111 minptime=10;useinbandfec=1',
      'a=rtcp-mux',
    ];

    for (final c in iceCandidates) {
      if (c is Map<String, dynamic>) {
        final foundation = c['foundation'] ?? '1';
        final priority = c['priority'] ?? 2130706431;
        final ip = c['ip'] ?? '127.0.0.1';
        final protocol = (c['protocol']?.toString() ?? 'udp').toLowerCase();
        final port = c['port'] ?? 40000;
        final type = c['type'] ?? 'host';
        lines.add(
            'a=candidate:$foundation 1 $protocol $priority $ip $port typ $type');
      }
    }

    return '${lines.join('\r\n')}\r\n';
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

    final offer = await _producerPeerConnection!.createOffer({
      'offerToReceiveAudio': false,
      'offerToReceiveVideo': false,
    });
    await _producerPeerConnection!.setLocalDescription(offer);

    // Extract real DTLS parameters from the offer SDP
    final dtlsParameters = extractDtlsParameters(offer.sdp!, role: 'client');

    // Connect producer transport with Mediasoup
    await _socketSession?.mutation('voice.connectProducerTransport', {
      'dtlsParameters': dtlsParameters,
    });

    // Complete SDP negotiation so ICE and DTLS connect on live networks
    final answerSdp = buildRemoteAnswerSdp(
      localSdp: offer.sdp!,
      transportParams: params,
    );
    await _producerPeerConnection!.setRemoteDescription(
      RTCSessionDescription(answerSdp, 'answer'),
    );

    // Call voice.produce with real extracted RTP parameters
    final rtpParameters = extractRtpParameters(offer.sdp!);
    final transportId = params['id']?.toString() ?? '';
    final produceResult = await _socketSession?.mutation('voice.produce', {
      'transportId': transportId,
      'kind': 'audio',
      'rtpParameters': rtpParameters,
    });

    _audioProducerId = produceResult?.toString();
  }

  Future<void> _initializeConsumerTransport(Map<String, dynamic> params) async {
    final config = <String, dynamic>{
      'iceServers': [
        {'urls': 'stun:stun.l.google.com:19302'},
      ],
      'sdpSemantics': 'unified-plan',
    };

    _consumerPeerConnection = await createPeerConnection(config);

    await _consumerPeerConnection!.addTransceiver(
      kind: RTCRtpMediaType.RTCRtpMediaTypeAudio,
      init: RTCRtpTransceiverInit(direction: TransceiverDirection.RecvOnly),
    );

    final offer = await _consumerPeerConnection!.createOffer({
      'offerToReceiveAudio': true,
      'offerToReceiveVideo': false,
    });
    await _consumerPeerConnection!.setLocalDescription(offer);

    // Extract real DTLS parameters from consumer offer SDP
    final dtlsParameters = extractDtlsParameters(offer.sdp!, role: 'client');

    // Connect consumer transport with Mediasoup
    await _socketSession?.mutation('voice.connectConsumerTransport', {
      'dtlsParameters': dtlsParameters,
    });

    // Complete SDP negotiation for consumer peer connection
    final answerSdp = buildRemoteAnswerSdp(
      localSdp: offer.sdp!,
      transportParams: params,
    );
    await _consumerPeerConnection!.setRemoteDescription(
      RTCSessionDescription(answerSdp, 'answer'),
    );

    // Handle incoming audio tracks
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

    // Route audio to device speakerphone by default
    try {
      await Helper.setSpeakerphoneOn(true);
    } catch (_) {}
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
          _previousSpeakingStates.remove(remoteId);
          onSpeakingChanged?.call(remoteId, false);
        }
      }
    });
  }

  Future<void> _consumeRemoteAudio(int remoteUserId) async {
    if (_consumerPeerConnection == null || _socketSession == null) return;

    try {
      final dynamic consumeResult =
          await _socketSession!.mutation('voice.consume', {
        'kind': 'audio',
        'remoteId': remoteUserId,
        'rtpCapabilities': {
          'codecs': [
            {
              'mimeType': 'audio/opus',
              'kind': 'audio',
              'clockRate': 48000,
              'channels': 2,
              'parameters': {
                'useinbandfec': 1,
                'usedtx': 1,
              },
            }
          ],
          'headerExtensions': [
            {
              'uri': 'urn:ietf:params:rtp-hdrext:sdes:mid',
              'id': 1,
            },
            {
              'uri': 'urn:ietf:params:rtp-hdrext:ssrc-audio-level',
              'id': 10,
            }
          ],
        },
      });

      if (consumeResult is Map<String, dynamic>) {
        final consumerId = consumeResult['consumerId']?.toString();
        if (consumerId != null) {
          _consumerToUserId[consumerId] = remoteUserId.toString();
        }
      }
    } catch (_) {
      // Consume error ignored
    }
  }

  Future<void> _cleanup() async {
    _stopStatsPolling();
    await _socketEventsSub?.cancel();
    _socketEventsSub = null;
    _consumerToUserId.clear();
    _previousSpeakingStates.clear();

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
