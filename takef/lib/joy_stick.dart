import 'package:flutter_joystick/flutter_joystick.dart';
import 'package:flutter/material.dart';

class ThumbStickController extends StatelessWidget{
  const ThumbStickController({super.key, this.onChange, this.size = 120, this.input = 0});
  final double size;
  final int input; //0 = mode.all, 1 = mode.Horizontal and Vertical
  final void Function(double x, double y)? onChange; //updates position joystick is moving in

  @override
  Widget build(BuildContext context){
    return Container(
      width: size, // same size to keep symmetry
      height: size,
      decoration: BoxDecoration(
        color: Colors.transparent,
        shape: BoxShape.rectangle,
      ),
      child: Joystick(
          mode: input == 0 ? JoystickMode.all
              :JoystickMode.horizontalAndVertical, //The directions the joystick move
          listener: (details){
            //details x and y range from -1.0 to 1.0
            onChange?.call(details.x, details.y);
          },
        base: JoystickBase(
          decoration: JoystickBaseDecoration(
            color: Colors.grey,
          ),
        ),
        stick: JoystickStick(
          decoration: JoystickStickDecoration(
            color: Colors.white
          ),
        ),
      ),
    );
  }
}