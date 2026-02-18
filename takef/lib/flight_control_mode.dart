import 'package:flutter/cupertino.dart';
import 'package:shared_preferences/shared_preferences.dart';

enum ControlMode {
  keyboard,
  joystick,
}

Future<void> saveControlMode(ControlMode mode) async{
  final prefs= await SharedPreferences.getInstance();
  //debugPrint("Saving mode = ${mode.name}");
  await prefs.setString('control_mode', mode.name);
}

Future<ControlMode> loadControlMode() async{
  final prefs = await SharedPreferences.getInstance();
  final value = prefs.getString('control_mode');
  //debugPrint("Loaded control_mode = $value");
  if(value == null) return ControlMode.joystick;

  return ControlMode.values.firstWhere(
      (e) => e.name == value,
    orElse: () => ControlMode.joystick,
  );
}