import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:network_info_plus/network_info_plus.dart';

import 'exception/not_connected_exception.dart';

class Sender {
  RawServerSocket? _socket;
  RawSocket? _connection;

  Stream<String> start() {
    final controller = StreamController<String>();
    NetworkInfo()
        .getWifiIP()
        .then((ip) {
          if (ip == null) {
            controller.addError(NotConnectedException());
            return;
          }
          RawServerSocket.bind(ip, 50551).then((socket) {
              sendStart(controller, ip);
              _socket = socket;

              socket.listen((event) {
                if (_connection == null) {
                  controller
                      .add("Client ${event.remoteAddress.address} connected");
                  _connection = event;
                  event.listen((event) {}, onDone: () {
                    _connection = null;
                    sendStart(controller, ip);
                    debugPrint("Connection closed");
                  }, onError: (err) {
                    _connection = null;
                    sendStart(controller, ip);
                    debugPrint(err);
                    debugPrint("Connection closed");
                  });
                } else {
                  controller.add(
                      "Client ${event.remoteAddress.address}:${event.port} tried to connect, but one connection was already established");
                }
              });
            });
        });

    return controller.stream;
  }

  Future<void> stop() async {
    await _connection?.close();
    await _socket?.close();
  }

  void sendStart(StreamController<String> controller, String? ip) {
    controller.add("Waiting connection on $ip:50551");
  }

  void sendData(Int16List data) {
    _connection?.write(data.buffer.asUint8List());
  }
}
