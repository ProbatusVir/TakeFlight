import 'dart:async';
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:takef/record_button.dart';
import 'package:takef/settings_screen.dart';
import 'package:takef/video_feed.dart';
import 'package:showcaseview/showcaseview.dart';
import 'connect.dart';
import 'flight_button.dart';
import 'central_screen.dart';
import 'joy_stick.dart';

///Global Key for first showcase widget
final GlobalKey _firstShow = GlobalKey();
///Global key for last showcase widget
final GlobalKey _lastShow = GlobalKey();
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
  ///Global key for each showcased widgets
  final GlobalKey _flightKey = GlobalKey();
  final GlobalKey _emergencyKey = GlobalKey();
  final GlobalKey _recordKey = GlobalKey();
  final GlobalKey _settingsKey = GlobalKey();

  double lr = 0;
  double fb = 0;
  double ud = 0;
  double rot = 0;

  void sendRC(){
    widget.rcCon.buildPacket(lr,ud,fb,rot);
    widget.control.sendRC();
  }
  
  @override
  void initState() {
    super.initState();
    ShowcaseView.register(
      hideFloatingActionWidgetForShowcase: [_lastShow],
      globalFloatingActionWidget: (showcaseContext) => FloatingActionWidget(
        left: 16,
          bottom: 16,
          child: Padding(
              padding: const EdgeInsets.all(16.0),
            child: ElevatedButton(
                onPressed: () => ShowcaseView.get().dismiss(),
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xffEE5366),
                ),
                child: const Text(
                  'Skip',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 15,
                  ),
                )
            ),
          )
      ),
      onStart: (index, key){
        log('onStart: $index, $key');
      },
      onComplete: (index, key){
        log('onComplete: $index, $key');
        if(index == 5){
          SystemChrome.setSystemUIOverlayStyle(
              SystemUiOverlayStyle.light.copyWith(
                statusBarBrightness: Brightness.dark,
                statusBarColor: Colors.white
              )
          );
        }
      },
      blurValue: 1,
      autoPlayDelay: const Duration(seconds: 3),
      globalTooltipActionConfig: const TooltipActionConfig(
        position: TooltipActionPosition.inside,
        alignment: MainAxisAlignment.spaceBetween,
        actionGap: 20,
      ),
      globalTooltipActions: [
        //Hide previous action for first showcase widget
        TooltipActionButton(
            type: TooltipDefaultActionType.previous,
          textStyle: const TextStyle(
            color: Colors.white,
          ),
          hideActionWidgetForShowcase: [_firstShow],
        ),
        //Same for last showcase
        TooltipActionButton(
            type: TooltipDefaultActionType.next,
          textStyle: const TextStyle(
            color: Colors.white
          ),
          hideActionWidgetForShowcase: [_lastShow],
        ),
      ],
      onDismiss: (key){
        debugPrint('Dismissed at $key');
      },
    );
    //Start showcase view
    WidgetsBinding.instance.addPostFrameCallback(
        (_) => ShowcaseView.get().startShowCase(
            [_firstShow, _flightKey, _emergencyKey, _recordKey, _settingsKey, _lastShow]
        )
    );
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
                    Showcase(key: _flightKey, title: 'Flight Control', description: 'Start or Stop drone flight', child: FlightButton(control: widget.control, size: 50.0,)),
                    Showcase(
                        key: _emergencyKey,
                        title: 'Emergency Landing',
                        description: 'Immediately cuts the drone off',
                        child: Tooltip(
                          message: "Emergency Land",
                          child: IconButton(
                              onPressed: (){
                                widget.control.sendLanding(0x02);
                              },
                              icon: Icon(Icons.emergency_sharp, color: Colors.red, size: 50.0,)
                          ),
                        ),
                    ),
                    //TODO::Implement actual recording logic here
                    Showcase(key: _recordKey, title: 'Record/Stop Recording', description: 'Records the live feed of the drone', child: RecordButton(getFrames: () => widget.videoKey.currentState?.currentFrame,)),
                    Showcase(
                        key: _settingsKey,
                        title: 'Settings',
                        description: 'Configure Controls & Personalize the application',
                        child: Tooltip(
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
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Padding(
                          padding: const EdgeInsets.only(left: 32, bottom: 32),
                          child: Showcase(
                              key: _firstShow,
                              title: 'Movement',
                              description: 'Controls forward, backward, and diagonal movement of the drone',
                              child: ThumbStickController(
                                onChange: (x, y){
                                  //Will be movement logic here
                                  lr = x;
                                  fb = y;
                                  sendRC();
                                },
                              ),
                          )
                        ),
                        Text(
                          "Movement"
                        )
                      ],
                    )
                  ),
                  Align( //Joy sticks bottom right
                    alignment: Alignment.bottomRight,
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Padding(
                          padding: const EdgeInsets.only(right: 32, bottom: 32),
                          child: Showcase(
                              key: _lastShow,
                              title: 'Height & YAW',
                              description: 'Controls drones rotation and height',
                              child: ThumbStickController(
                                onChange: (x, y){
                                  //Will be height/axis control logic
                                  rot = x;
                                  ud = y;
                                  sendRC();
                                },
                              ),
                          )
                        ),
                        Text(
                          "Height & Rotation"
                        )
                      ],
                    )
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