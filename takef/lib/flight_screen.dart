import 'package:flutter/material.dart';
import 'record_button.dart';
import 'package:flutter/services.dart';
import 'joy_stick.dart';
import 'settings_screen.dart';
import 'video_feed.dart';
import 'connect.dart';

class RC{
  List<int> packet = [];

  List<int> buildPacket(double lr, double ud, double fb, double rot){
    //multiply by 100 for -100.0 to 100;
    final leftRight = (lr * 100).toInt();
    final upDown = (ud * 100).toInt();
    final forwardBack = (fb * 100).toInt();
    final rotation = (rot * 100).toInt();

    return packet = [
      0x02, //Rc command code
      leftRight.toSigned(8),
      upDown.toSigned(8),
      forwardBack.toSigned(8),
      rotation.toSigned(8),
      0x00 //Reserved
    ];
  }
}

final rcCon = RC();

bool isMobile(BuildContext context){
  bool mob = false;
  final width = MediaQuery.of(context).size.width;
  final height = MediaQuery.of(context).size.height;
  final or = MediaQuery.of(context).orientation;
  print('Width:$width \n\n Height: $height \n\n Orientation: $or');

  if(width <= 800){
    mob = true;
  }
  return mob;
}

class FlightScreen extends StatefulWidget{
  const FlightScreen({super.key});

  @override
  State<FlightScreen> createState() => _FlightScreenState();
}
class _FlightScreenState extends State<FlightScreen>{
  //create a global key to access the curentFrame variable in video feed
  final GlobalKey<VideoFeedState> videoKey = GlobalKey();

  /*static void rc(double lr, double ud, double fb, double rot) async{
    //TODO::Change to future async once server connection is there
    final rcController = RC();
    rcController.buildPacket(lr, ud, fb, rot);
    //simulate movement to drone til connection to server is established
    //print('RC Commands: left/right$lr:up/down$ud:forward/backward$fb:rotation$rot'); //debug
  }*/

  //Changed to stateful widget to force this screen into landscape for android
  @override
  void initState(){
    super.initState();
    SystemChrome.setPreferredOrientations([
      DeviceOrientation.landscapeLeft,
      DeviceOrientation.landscapeRight,
    ]);
  }
  @override
  void dispose(){
    super.dispose();
    SystemChrome.setPreferredOrientations([
      DeviceOrientation.landscapeLeft,
      DeviceOrientation.landscapeRight,
      DeviceOrientation.portraitUp,
      DeviceOrientation.portraitDown
    ]);
  }
  @override
  Widget build(BuildContext context) {
    return isMobile(context) ? MobileFlight(videoKey: videoKey,) : DeskFlight(videoKey: videoKey);
  }
}

class MobileFlight extends StatelessWidget{
  const MobileFlight({
    super.key,
    required this.videoKey
  });

  final GlobalKey<VideoFeedState> videoKey;

  @override
  Widget build(BuildContext context) {
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
    );
  }
}

class DeskFlight extends StatelessWidget {
  const DeskFlight({
    super.key,
    required this.videoKey,
  });

  final GlobalKey<VideoFeedState> videoKey;

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
          Center(child: VideoFeed(key: videoKey,),),//placement for video feed and to record it
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
                    IconButton(
                        onPressed: (){},
                        icon: Icon(Icons.flight_takeoff, color: Colors.white, size: 50.0,)
                    ),
                    //TODO::Implement actual recording logic here
                    RecordButton(getFrames: () => videoKey.currentState?.currentFrame,),
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
              padding: const EdgeInsets.all(100),
              child: ThumbStickController(
                onChange: (x, y){
                  //Will be movement logic here
                  final lr = x;
                  final fb = y;
                  rcCon.buildPacket(lr,0,fb,0);
                  sendRC();
                },
              ),
            ),
          ),
          Align( //Joy sticks bottom right
            alignment: Alignment.bottomRight,
            child: Padding(
              padding: const EdgeInsets.all(100),
              child: ThumbStickController(
                input: 1,
                onChange: (x, y){
                  //Will be height/axis control logic
                  final rot = x;
                  final ud = y;
                  rcCon.buildPacket(0,ud,0,rot);
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