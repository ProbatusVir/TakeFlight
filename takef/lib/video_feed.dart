import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'connect.dart';

class VideoFeed extends StatefulWidget{
  const VideoFeed({super.key});

  @override
  State<VideoFeed> createState() => VideoFeedState();
}

class VideoFeedState extends State<VideoFeed>{
  /*late Timer timer;
  int frame = 0;
  late List<Image> feed;
  bool isLoad = false;*/
  List<Uint8List> currentFrame = [];
  Uint8List? latestFrame;

  @override
  void initState(){
    super.initState();
    onImageReceived();
  }

  void onImageReceived() async {
    final vid = DroneVideo();
    await vid.connect();
    await vid.getDroneImg(latestFrame!);
    currentFrame.add(latestFrame!);
    /*setState(() {
      print('VideoFeedState received frame of ${received.length} bytes');
      latestFrame = received;
      currentFrame.add(latestFrame!);
    });*/
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