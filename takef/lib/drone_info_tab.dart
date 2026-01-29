import 'package:flutter/material.dart';

import 'connect.dart';

class DroneInfoPage extends StatefulWidget{
  const DroneInfoPage({super.key, required this.info});
  final Map<String, dynamic> info;

  @override
  State<DroneInfoPage> createState() => _DroneInfoPageState();
}

class _DroneInfoPageState extends State<DroneInfoPage>{

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Align(
          alignment: Alignment.topRight,
          child: IconButton(
            icon: Icon(Icons.arrow_back),
            color: Colors.white,
            onPressed: (){
              Navigator.of(context).pop();
            },
          ),
        ),
        Expanded(
          child: Row(
            children: [
              ///Icon column
              Expanded(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text('Drone1:',
                        style: TextStyle(
                          color: Colors.white,
                          fontWeight: FontWeight.bold,
                          fontSize: 30.0,
                        ),
                      ),
                      Icon(
                        Icons.battery_4_bar,
                        size: 25.0 ,
                        color: Colors.green,
                      ),
                      Icon(
                        Icons.thermostat,
                        size: 25.0 ,
                        color: Colors.white,
                      ),
                      Icon(
                        Icons.access_time,
                        size: 25.0 ,
                        color: Colors.white,
                      ),
                    ],
                  ),
              ),
              ///Details column
              Expanded(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Battery Status: 67%',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 30.0,
                      ),
                    ),
                    Text(
                      'Temperature: 35°C',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 30.0,
                      ),
                    ),
                    Text(
                      'Flight Time: 12m 34s',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 30.0,
                      ),
                    ),
                    Text(
                      'Description:',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 30.0,
                      ),
                    ),
                    Text(
                      'Model Info: ...',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 20.0,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}