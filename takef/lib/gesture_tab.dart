import 'package:flutter/material.dart';

class GestureControlPage extends StatefulWidget{
  const GestureControlPage({super.key});

  @override
  State<StatefulWidget> createState() => _GestureControlPageState();
}

class _GestureControlPageState extends State<GestureControlPage>{

  @override
  Widget build(BuildContext context) {
    return  Center(
      child: Text('Gesture Settings content'),
    );
  }
}