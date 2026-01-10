import 'package:flutter/material.dart';
import 'connect.dart';

class FlightButton extends StatefulWidget{
  const FlightButton({super.key, required this.control, this.size = 30.0});
  final ControlRC control;
  final double size;

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
          icon: Icon(Icons.flight_land, color: Colors.white, size: widget.size,)
      ) :
        IconButton(
            onPressed: (){
              widget.control.sendTakeOff();
              setState(() {
                isFlying = true;
              });
            },
            icon: Icon(Icons.flight_takeoff, color: Colors.white, size: widget.size,)
        )
    );
  }
}