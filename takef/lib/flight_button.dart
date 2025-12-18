import 'package:flutter/material.dart';
import 'connect.dart';

class FlightButton extends StatefulWidget{
  const FlightButton({super.key, required this.control});
  final ControlRC control;

  @override
  State<FlightButton> createState() => _FlightButtonState();
}

class _FlightButtonState extends State<FlightButton>{
  bool isFlying = false;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      child: isFlying ? IconButton(
          onPressed: (){
            widget.control.sendLanding(0x01);
            setState(() {
              isFlying = false;
            });
          },
          icon: Icon(Icons.flight_land, color: Colors.white, size: 30.0,)
      ) :
        IconButton(
            onPressed: (){
              widget.control.sendTakeOff();
              setState(() {
                isFlying = true;
              });
            },
            icon: Icon(Icons.flight_takeoff, color: Colors.white, size: 30.0,)
        )
    );
  }
}