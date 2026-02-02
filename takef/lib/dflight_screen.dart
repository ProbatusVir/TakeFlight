import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:takef/record_button.dart';
import 'package:takef/settings_screen.dart';
import 'package:takef/video_feed.dart';
import 'connect.dart';
import 'flight_button.dart';
import 'central_screen.dart';
import 'joy_stick.dart';

///Desktop Design
class DeskFlight extends StatefulWidget{
  const DeskFlight({
    super.key,
    required this.videoKey,
    required this.port,
    required this.control,
    required this.rcCon,
    required this.info
  });

  final GlobalKey<VideoFeedState> videoKey;
  final int port;
  final ControlRC control;
  final RC rcCon;
  final Map<String, dynamic> info;

  @override
  State<DeskFlight> createState() => _DeskFlightState();
}

class _DeskFlightState extends State<DeskFlight>{
  final Set<LogicalKeyboardKey> keys ={};

  double lr = 0;
  double fb = 0;
  double ud = 0;
  double rot = 0;

  void sendRC(){
    widget.rcCon.buildPacket(lr,ud,fb,rot);
    widget.control.sendRC();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      extendBodyBehindAppBar: true,
      backgroundColor: Colors.grey,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        elevation: 0.0, //gets rid of the shadow
        title: Text('H: 48m', style: TextStyle(color: Colors.white, fontSize: 25.0),),
        actions: [
          Icon(Icons.wifi_outlined, color: Colors.white, size: 35.0,),
          SizedBox(width: 50,), //for spacing between objects
          Text('12m 34s', style: TextStyle(color: Colors.white, fontSize: 25.0),),
          SizedBox(width: 50,),
          Icon(Icons.battery_6_bar_sharp, color: Colors.green,size: 35.0,),
        ],
      ),
      body: Stack(
        children: [
          Center(child: VideoFeed(key: widget.videoKey,port: widget.port,),),//placement for video feed and to record it
          Align( //Aligns user menu to bottom center of the screen
            alignment: Alignment.bottomCenter,
            child: Padding(
              padding: const EdgeInsets.only(bottom: 30), //spacing needed so it isn't touching bottom of the screen
              child: Container( //The actual circular box
                decoration: BoxDecoration(
                    color: Colors.black,
                    borderRadius: BorderRadius.circular(50)
                ),
                child: Row( //Organization of Icons in the box
                  mainAxisAlignment: MainAxisAlignment.spaceEvenly, //Spaces widgets evenly within the Row
                  mainAxisSize: MainAxisSize.min, //Minimal size needed to fit
                  children: [
                    FlightButton(control: widget.control, size: 50.0,),
                    Tooltip(
                      message: "Emergency Land",
                      child: IconButton(
                          onPressed: (){
                            widget.control.sendLanding(0x02);
                          },
                          icon: Icon(Icons.emergency_sharp, color: Colors.red, size: 50.0,)
                      ),
                    ),
                    //TODO::Implement actual recording logic here
                    RecordButton(getFrames: () => widget.videoKey.currentState?.currentFrame,),
                    Tooltip(
                      message: "Settings",
                      child: IconButton(
                          onPressed: () {
                            Navigator.of(context).push(
                                MaterialPageRoute(builder: (BuildContext context) => Settings(info: widget.info,))
                            );
                          },
                          icon: Icon(Icons.settings_outlined, color: Colors.white, size: 50.0,)
                      ),
                    )
                  ],
                ),
              ),
            ),
          ),
          //add keyboard listener
          Focus(
            autofocus: true,
              onKeyEvent: _onKey,
              child: Stack(
                children: [
                  Align( //Joy sticks bottom left
                    alignment: Alignment.bottomLeft,
                    child: Padding(
                      padding: const EdgeInsets.all(125),
                      child: ThumbStickController(
                        onChange: (x, y){
                          //Will be movement logic here
                          lr = x;
                          fb = y;
                          sendRC();
                        },
                      ),
                    ),
                  ),
                  Align( //Joy sticks bottom right
                    alignment: Alignment.bottomRight,
                    child: Padding(
                      padding: const EdgeInsets.all(125),
                      child: ThumbStickController(
                        onChange: (x, y){
                          //Will be height/axis control logic
                          rot = x;
                          ud = y;
                          sendRC();
                        },
                      ),
                    ),
                  ),
                ],
              ),
          )
        ],
      ),
    );
  }
  double analog(double current, double target, double rate){
    return current += (target - current) * rate;
}
  KeyEventResult _onKey(FocusNode node, KeyEvent event){
    //Track keys being pressed/released
    if(event is KeyDownEvent){
      keys.add(event.logicalKey);
    }else if (event is KeyUpEvent){
      keys.remove(event.logicalKey);
    }else{
      return KeyEventResult.ignored;
    }
    //testing what adding a periodic timer does
    Timer.periodic(const Duration( milliseconds: 20), (_){
      ///Movement (Left Stick)
      double targetLr = (keys.contains(LogicalKeyboardKey.keyD) ? 1 : 0) +
          (keys.contains(LogicalKeyboardKey.keyA) ? -1 : 0);

      double targetFb = (keys.contains(LogicalKeyboardKey.keyW) ? 1 : 0) +
          (keys.contains(LogicalKeyboardKey.keyS) ? -1 : 0);

      lr = analog(lr, targetLr, 0.02);
      fb = analog(fb, targetFb, 0.02);

      ///Altitude & Rotation (Right Stick)
      double targetRot = (keys.contains(LogicalKeyboardKey.keyQ) ? 1 : 0) +
          (keys.contains(LogicalKeyboardKey.keyE) ? -1 : 0);

      double targetUd = (keys.contains(LogicalKeyboardKey.arrowUp) ? 1 : 0) +
          (keys.contains(LogicalKeyboardKey.arrowDown) ? -1 : 0);

      rot = analog(rot, targetRot, 0.02);
      ud = analog(ud, targetUd, 0.02);

      sendRC();
    });

    return KeyEventResult.handled;
  }
}