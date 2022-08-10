import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/services.dart';

class Subscriber {
  static const _channel = MethodChannel('fast_mic/recording');

  Timer? _timerSubscription;

  StreamController<Int16List>? _controller;

  Stream<Int16List> startRecording() {
    _controller?.close();
    _controller = StreamController();
    if (_timerSubscription == null) {
      _channel.invokeMethod("start").then((result) {
        _timerSubscription =
            Timer.periodic(const Duration(milliseconds: 150), (timer) async {
          final buffer = await _channel.invokeMethod("poll");
          _publish(buffer);
        });
      });
    }
    return _controller!.stream;
  }

  Future<void> stopRecording() async {
    if (_timerSubscription != null) {
      _timerSubscription?.cancel();
      await _channel.invokeMethod("stop");
      await _controller?.close();
      _controller = null;
      _timerSubscription = null;
    }
  }

  void _publish(dynamic event) {
    final list = (event as List<Object?>).map((e) => e! as int).toList();
    _controller!.add(Int16List.fromList(list));
  }
}
