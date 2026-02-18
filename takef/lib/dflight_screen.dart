import 'dart:async';
import 'dart:developer';

import 'package:battery_indicator/battery_indicator.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:takef/flight_control_mode.dart';
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
  bool isPressed(LogicalKeyboardKey key) =>
      keys.contains(key);
  ///Global key for each showcased widgets
  final GlobalKey _flightKey = GlobalKey();
  final GlobalKey _emergencyKey = GlobalKey();
  final GlobalKey _recordKey = GlobalKey();
  final GlobalKey _settingsKey = GlobalKey();

  double lr = 0;
  double fb = 0;
  double ud = 0;
  double rot = 0;

  ControlMode mode = ControlMode.joystick;

  void sendRC(){
    widget.rcCon.buildPacket(lr,ud,fb,rot);
    widget.control.sendRC();
  }

  ///Keyboard display
  Widget keyBox(String label, LogicalKeyboardKey key) {
    final pressed = isPressed(key);

    return AnimatedContainer(
      duration: Duration(milliseconds: 100),
      width: 60,
      height: 60,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: pressed ? Colors.blue : Colors.grey[800],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.white24),
      ),
      child: Text(
        label,
        style: TextStyle(fontSize: 20, color: Colors.white),
      ),
    );
  }
  
  @override
  void initState() {
    super.initState();
    loadControlMode().then((loadedMode){
      setState(() {
        mode = loadedMode;
      });
    });
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
        title: Text(widget.info.isEmpty ? "H: 0" : "H:${widget.info["height"]}", style: TextStyle(color: Colors.white, fontSize: 25.0),),
        actions: [
          Icon(Icons.wifi_outlined, color: Colors.white, size: 35.0,),
          SizedBox(width: 50,), //for spacing between objects
          Text(widget.info.isEmpty ?  "D: 0m:00sec" : "D: ${widget.info["flight_duration"]}", style: TextStyle(color: Colors.white, fontSize: 25.0),),
          SizedBox(width: 50,),
          //Icon(Icons.battery_6_bar_sharp, color: Colors.green,size: 35.0,),
          BatteryIndicator(
            batteryFromPhone: false,
            batteryLevel: widget.info.isEmpty ? 67 : widget.info['battery'],
            colorful: true,
            mainColor: Colors.green,
            size: 35.0,
            showPercentNum: true,
          )
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
                  Align( //Joy sticks bottom left & WASD movement keys
                    alignment: Alignment.bottomLeft,
                    child: mode == ControlMode.keyboard
                        ? Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        keyBox("W", LogicalKeyboardKey.keyW),
                        SizedBox(height: 6),
                        Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            keyBox("A", LogicalKeyboardKey.keyA),
                            SizedBox(height: 6),
                            keyBox("S", LogicalKeyboardKey.keyS),
                            SizedBox(height: 6),
                            keyBox("D", LogicalKeyboardKey.keyD)
                          ],
                        )
                      ],
                    ): Column(
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
                  Align( //Joy sticks bottom right & Arrow keys
                    alignment: Alignment.bottomRight,
                    child: mode == ControlMode.keyboard
                        ? Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        keyBox("↑", LogicalKeyboardKey.arrowUp),
                        SizedBox(height: 6),
                        Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            keyBox("←", LogicalKeyboardKey.arrowLeft),
                            SizedBox(height: 6),
                            keyBox("↓", LogicalKeyboardKey.arrowDown),
                            SizedBox(height: 6),
                            keyBox("→", LogicalKeyboardKey.arrowRight)
                          ],
                        )
                      ],
                    ): Column(
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
      double targetRot = (keys.contains(LogicalKeyboardKey.arrowLeft) ? 1 : 0) +
          (keys.contains(LogicalKeyboardKey.arrowRight) ? -1 : 0);

      double targetUd = (keys.contains(LogicalKeyboardKey.arrowUp) ? 1 : 0) +
          (keys.contains(LogicalKeyboardKey.arrowDown) ? -1 : 0);

      rot = analog(rot, targetRot, 0.02);
      ud = analog(ud, targetUd, 0.02);

      sendRC();
    });

    return KeyEventResult.handled;
  }
}