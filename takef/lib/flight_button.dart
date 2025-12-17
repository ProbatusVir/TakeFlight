import 'package:flutter/material.dart';
import 'connect.dart';

class FlightButton extends StatefulWidget{
  const FlightButton({super.key});

  @override
  State<FlightButton> createState() => _FlightButtonState();
}

class _FlightButtonState extends State<FlightButton>{
  bool isFlying = false;
  final control = ControlRC();

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: (){
        setState(() {
          isFlying = !isFlying;
        });
      },
      child: isFlying ? IconButton(
          onPressed: (){
            control.sendTakeOff();
          },
          icon: Icon(Icons.flight_takeoff, color: Colors.white, size: 30.0,)
      ) :
      IconButton(
          onPressed: (){
            control.sendLanding(0x01);
          },
          icon: Icon(Icons.flight_land, color: Colors.white, size: 30.0,)
      ),
    );
  }
}