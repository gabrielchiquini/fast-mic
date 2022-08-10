import 'dart:async';
import 'dart:typed_data';

import 'package:fast_mic/exception/default_exception.dart';
import 'package:fast_mic/socket.dart';
import 'package:fast_mic/subscriber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_background/flutter_background.dart';

const androidConfig = FlutterBackgroundAndroidConfig(
  notificationTitle: "Fast Mic",
  notificationText: "Fast mic is listening",
  notificationImportance: AndroidNotificationImportance.Default,
);

// palette https://colorhunt.co/palette/0000003d0000950101ff0000
const red2 = Color(0xFF950101);
const red1 = Color(0xFF3D0000);

class RecordToStreamExample extends StatefulWidget {
  const RecordToStreamExample({Key? key}) : super(key: key);

  @override
  _RecordToStreamExampleState createState() => _RecordToStreamExampleState();
}

class _RecordToStreamExampleState extends State<RecordToStreamExample> {
  final _sender = Sender();
  final _subscriber = Subscriber();
  bool isRecording = false;
  String _statusText = "Ready";

  @override
  void initState() {
    super.initState();
  }

  @override
  void dispose() {
    stopRecorder();
    super.dispose();
  }

  void startRecorder() async {
    FlutterBackground.enableBackgroundExecution();
    _sender.start().listen((event) {
      setState(() {
        _statusText = event;
      });
    }).onError((err) {
      if (err is DefaultException) {
        setState(() {
          _statusText = err.message;
        });
      }
    });
    _subscriber.startRecording().listen((event) {
      _sender.sendData(event);
    });

    setState(() {
      isRecording = true;
    });
  }

  Future<void> stopRecorder() async {
    FlutterBackground.disableBackgroundExecution();
    await _subscriber.stopRecording();
    await _sender.stop();
    setState(() {
      isRecording = false;
      _statusText = "Stopped";
    });
  }

  @override
  Widget build(BuildContext context) {
    Widget makeBody() {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.max,
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 15, vertical: 10),
              child: Text(_statusText),
              alignment: AlignmentDirectional.center,
              width: double.infinity,
            ),
            const SizedBox(height: 20),
            ElevatedButton(
              onPressed: isRecording ? stopRecorder : startRecorder,
              child: Icon(
                isRecording ? Icons.stop : Icons.play_arrow,
                size: 40,
              ),
              style: ElevatedButton.styleFrom(
                shape: const CircleBorder(),
                padding: const EdgeInsets.all(16),
              ),
            ),
          ],
        ),
      );
    }

    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        title: const Text('Fast Mic'),
        elevation: 0,
      ),
      body: makeBody(),
    );
  }
}

class MyApp extends StatelessWidget {
  const MyApp({Key? key}) : super(key: key);

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    FlutterBackground.initialize(androidConfig: androidConfig);
    return MaterialApp(
      title: 'Flutter Demo',
      home: const RecordToStreamExample(),
      themeMode: ThemeMode.dark,
      darkTheme: ThemeData.dark().copyWith(
        backgroundColor: Colors.black,
        canvasColor: Colors.black,
        primaryColor: red2,
        appBarTheme: AppBarTheme.of(context).copyWith(
          backgroundColor: red1,
        ),
        elevatedButtonTheme: ElevatedButtonThemeData(
          style: ButtonStyle(
            elevation: MaterialStateProperty.all(0),
            backgroundColor: MaterialStateProperty.all(red2),
          ),
        ),
      ),
    );
  }
}

void main() {
  runApp(const MyApp());
}

void lowPass(Int16List bytes) {
  var lastSample = 0;
  for (var i = 0; i < bytes.length; i++) {
    final sample = bytes[i];
    final input = (sample + (lastSample * 7)) >> 3;
    lastSample = input;
    bytes[i] = input;
  }
}
