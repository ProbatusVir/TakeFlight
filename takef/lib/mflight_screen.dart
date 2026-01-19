import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:takef/record_button.dart';
import 'package:takef/video_feed.dart';
import 'connect.dart';
import 'flight_button.dart';
import 'central_screen.dart';
import 'joy_stick.dart';
import 'msettings_screen.dart';

///Mobile Design
class MobileFlight extends StatefulWidget{
  const MobileFlight({
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
  State<MobileFlight> createState() => _MobileFlightState();
}

class _MobileFlightState extends State<MobileFlight>{

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
    SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky); //hides the nav and status bar
    return Scaffold(
      extendBodyBehindAppBar: true,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        elevation: 0.0, //gets rid of the shadow
        title: Text('H: 45m', style: TextStyle(color: Colors.white, fontSize: 15.0),),
        actions: [
          Icon(Icons.wifi_outlined, color: Colors.white, size: 25.0,),
          SizedBox(width: 50,), //for spacing between objects
          Text('12m 34s', style: TextStyle(color: Colors.white, fontSize: 15.0),),
          SizedBox(width: 50,),
          Icon(Icons.battery_6_bar_sharp, color: Colors.green,size: 25.0,),
        ],
      ),
      body: Stack(
        children: [
          Align(
            alignment: Alignment.center,
            //TODO:: Need to fix weird visual bug of the feed being small then readjusting to correct size
            child: VideoFeed(key: widget.videoKey,port: widget.port,),
          ),
          Align(
            alignment: Alignment.bottomCenter,
            child: Padding(
              padding: const EdgeInsets.only(bottom: 20),
              child: Container(
                decoration: BoxDecoration(
                    color: Colors.black,
                    borderRadius: BorderRadius.circular(30)
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceEvenly, //Spaces widgets evenly within the Row
                  mainAxisSize: MainAxisSize.min, //Minimal size needed to fit
                  children: [
                    FlightButton(control: control,),
                    RecordButton(getFrames: () => widget.videoKey.currentState?.currentFrame,),
                    IconButton(
                        onPressed: () {
                          Navigator.of(context).push(
                              MaterialPageRoute(builder: (BuildContext context) => MsettingsScreen())
                          );
                        },
                        icon: Icon(Icons.settings_outlined, color: Colors.white, size: 30.0,)
                    ),
                  ],
                ),
              ),
            ),
          ),
          Align( //Joy sticks bottom left
            alignment: Alignment.bottomLeft,
            child: Padding(
              padding: const EdgeInsets.all(50),
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
              padding: const EdgeInsets.all(50),
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