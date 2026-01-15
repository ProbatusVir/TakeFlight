import 'package:flutter/material.dart';

class FlightLogsPage extends StatefulWidget{
  const FlightLogsPage({super.key});

  @override
  State<FlightLogsPage> createState() => _FlightLogsPageState();
}

class _FlightLogsPageState extends State<FlightLogsPage>{

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Text('Flight Logs'),
    );
  }
}