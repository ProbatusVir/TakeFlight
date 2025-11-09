import 'dart:async';
import 'package:flutter/material.dart';

class VideoFeed extends StatefulWidget{
  const VideoFeed({super.key});

  @override
  State<VideoFeed> createState() => _VideoFeedState();
}

class _VideoFeedState extends State<VideoFeed>{
  late Timer timer;
  int frame = 0;
  late List<String> feed;

  //on creation of the state sets the list time and frame
  @override void initState() {
    super.initState();
    feed = List.generate(200, (step) => 'assets/simulated_feed/ezgif-frame-${(step + 1).toString().padLeft(3, '0')}.jpg');
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
    return Image.asset(feed[frame], fit: BoxFit.fill); //This will turn into a Image.memory
  }
}