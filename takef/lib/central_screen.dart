import 'package:flutter/material.dart';
import 'dflight_screen.dart';
import 'mflight_screen.dart';
import 'package:flutter/services.dart';
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

    packet = [
      0x02, //Rc command code
      leftRight.toSigned(8),
      upDown.toSigned(8),
      forwardBack.toSigned(8),
      rotation.toSigned(8),
      0x00 //Reserved
    ];
    //print('Current movement packet: $packet');

    return packet;
  }
}

final rcCon = RC();
final control = ControlRC();
final vid = DroneVideo();

bool isMobile(BuildContext context){
  bool mob = false;
  final width = MediaQuery.of(context).size.width;
  final height = MediaQuery.of(context).size.height;
  final or = MediaQuery.of(context).orientation;
  //print('Width:$width \n\n Height: $height \n\n Orientation: $or');

  if(width <= 800){
    mob = true;
  }
  return mob;
}

class FlightScreen extends StatefulWidget{
  const FlightScreen({super.key,required this.port, required this.info});
  final int port;
  final Info info;

  @override
  State<FlightScreen> createState() => _FlightScreenState();
}
class _FlightScreenState extends State<FlightScreen>{
  final GlobalKey<VideoFeedState> videoKey = GlobalKey<VideoFeedState>();

  //Changed to stateful widget to force this screen into landscape for android
  @override
  void initState(){
    super.initState();
    SystemChrome.setPreferredOrientations([
      DeviceOrientation.landscapeLeft,
      DeviceOrientation.landscapeRight,
    ]);
    startConnection();
  }

  void startConnection() async{
    await control.connect(0x01, widget.port);
    await vid.connect(widget.port, videoKey);
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
    return isMobile(context) ?
    MobileFlight(videoKey: videoKey, port: widget.port, control: control, rcCon: rcCon, info: widget.info,)
        :
    DeskFlight(videoKey: videoKey, port: widget.port, control: control, rcCon: rcCon, info: widget.info,);
  }
}
