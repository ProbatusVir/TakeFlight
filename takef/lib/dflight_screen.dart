import 'package:flutter/material.dart';
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
    required this.rcCon
  });

  final GlobalKey<VideoFeedState> videoKey;
  final int port;
  final ControlRC control;
  final RC rcCon;

  @override
  State<DeskFlight> createState() => _DeskFlightState();
}

class _DeskFlightState extends State<DeskFlight>{

  double lr = 0;
  double fb = 0;
  double ud = 0;
  double rot = 0;

  void sendRC(){
    widget.rcCon.buildPacket(lr,0,fb,0);
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
                    IconButton(
                        onPressed: (){
                          widget.control.sendLanding(0x02);
                        },
                        icon: Icon(Icons.emergency_sharp, color: Colors.red, size: 50.0,)
                    ),
                    //TODO::Implement actual recording logic here
                    RecordButton(getFrames: () => widget.videoKey.currentState?.currentFrame,),
                    IconButton(
                        onPressed: () {
                          Navigator.of(context).push(
                              MaterialPageRoute(builder: (BuildContext context) => Settings())
                          );
                        },
                        icon: Icon(Icons.settings_outlined, color: Colors.white, size: 50.0,)
                    ),
                  ],
                ),
              ),
            ),
          ),
          //TODO::Fix joystick UI and handling for android
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
                input: 1,
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
    );
  }
}