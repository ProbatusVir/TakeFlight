import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class VideoFeed extends StatefulWidget{
  const VideoFeed({super.key});

  @override
  State<VideoFeed> createState() => _VideoFeedState();
}

class _VideoFeedState extends State<VideoFeed>{
  late Timer timer;
  int frame = 0;
  late List<Image> feed;
  bool isLoad = false;

  //on creation of the state sets the list time and frame
  @override void initState() {
    super.initState();
    loadFeed().then((_){
      isLoad = true;
      //setState(() {});
      start();
    });
  }
  //separate them into functions
  Future<void> loadFeed() async{
    feed = [];
    //loop through feed
    for(var i = 1; i <= 200; i++){
      //wait to load all images into memory
      final bytes = await rootBundle.load('assets/simulated_feed/ezgif-frame-${i.toString().padLeft(3, '0')}.jpg');
      feed.add(Image.memory(bytes.buffer.asUint8List(), gaplessPlayback: true,));
    }
  }

  void start() {
    timer = Timer.periodic(const Duration(milliseconds: 35), (_) {
      setState(() {
        frame = (frame + 1) % feed.length;
      });
    });
  }

  @override
  void dispose() {
    timer.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    //will place a loading screen if false otherwise will show video feed
    return isLoad == false ? CircularProgressIndicator()
      : feed[frame];
  }
}