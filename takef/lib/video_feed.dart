import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:takef/disconnect.dart';
import 'connect.dart';

enum FeedState{
  connecting,
  live,
  timeout,
  disconnected,
}

class VideoFeed extends StatefulWidget{
  const VideoFeed({super.key, required this.port});
  final int port;

  @override
  State<VideoFeed> createState() => VideoFeedState();
}

class VideoFeedState extends State<VideoFeed>{
  List<Uint8List> currentFrame = [];
  Uint8List? latestFrame;
  FeedState _feedState = FeedState.connecting;
  Timer? _timeOut;

  @override
  void initState(){
    super.initState();
  }

  void resetTimeOut(){
    //Clear timeout timer
    _timeOut?.cancel();
    //Reset it at 3 seconds
    _timeOut = Timer(const Duration(seconds: 3), (){
      if(mounted){
        setState(() {
          //set the state to be in timeout
          _feedState = FeedState.timeout;
        });
      }
    });
  }

  void onImageReceived(Uint8List frames) async {
    setState(() {
      print('VideoFeedState received frame of ${frames.length} bytes');
      latestFrame = frames;
      currentFrame.add(latestFrame!);
      if(_feedState != FeedState.live && latestFrame != null){
        _feedState = FeedState.live;
      }
    });
    resetTimeOut();
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
    return Stack(
      children: [
        if(_feedState == FeedState.live && latestFrame != null)
          Image.memory(
            latestFrame!,
            gaplessPlayback: true,
            fit: BoxFit.cover,
            width: MediaQuery.of(context).size.width,
            height: MediaQuery.of(context).size.height,
          ),
        if(_feedState == FeedState.connecting)
          const CircularProgressIndicator(),
        if(_feedState == FeedState.timeout)
          noFeedIndicator(),
        ///TODO::Place disconnection logic here for flight screen
        /*if(_feedState == FeedState.disconnected)
          disconnect(context),*/
      ],
    );
  }
}

Widget noFeedIndicator(){
  return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.black.withOpacity(0.7),
        borderRadius: BorderRadius.circular(10),
      ),
      child: const Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.videocam_off, color: Colors.white),
          SizedBox(height: 8),
          Text(
            "No Video Feed",
            style: TextStyle(color: Colors.white),
          ),
        ],
      )
  );
}