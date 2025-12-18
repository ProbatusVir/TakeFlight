import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'connect.dart';

class VideoFeed extends StatefulWidget{
  const VideoFeed({super.key, required this.port});
  final int port;

  @override
  State<VideoFeed> createState() => VideoFeedState();
}

class VideoFeedState extends State<VideoFeed>{
  List<Uint8List> currentFrame = [];
  Uint8List? latestFrame;

  @override
  void initState(){
    super.initState();
  }

  void onImageReceived(Uint8List frames) async {
    setState(() {
      print('VideoFeedState received frame of ${frames.length} bytes');
      latestFrame = frames;
      currentFrame.add(latestFrame!);
    });
    /*final received = await vid.getDroneImg();
    if (received != null) {
      latestFrame = received;
      currentFrame.add(latestFrame!);
      print('VideoFeedState received frame of ${received.length} bytes');
    }*/
  }

  @override
  Widget build(BuildContext context) {
    //will place a loading screen if false otherwise will show video feed
    return latestFrame == null ? CircularProgressIndicator()
      : Image.memory(
      latestFrame!,
      gaplessPlayback: true,
      fit: BoxFit.cover,
      width: MediaQuery.of(context).size.width,
      height: MediaQuery.of(context).size.height,
    );
  }
}