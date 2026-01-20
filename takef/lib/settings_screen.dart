import 'package:flutter/material.dart';
import 'package:takef/drone_info_tab.dart';
import 'package:takef/flight_logs_tab.dart';
import 'package:takef/gesture_tab.dart';
import 'package:takef/personalization_tab.dart';
import 'connect.dart';

class Settings extends StatelessWidget{
  const Settings({super.key, required this.info});
  final Info info;

  @override
  Widget build(BuildContext context){
    return DefaultTabController(
        length: 4,
        child: Scaffold(
          backgroundColor: Colors.grey.shade700,
          body: Row(
            children: [
              //Left side tab bar
              Container(
                width: 125,
                height: double.maxFinite,
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.only(
                    topRight: Radius.circular(20),
                    bottomRight: Radius.circular(20),
                  ),
                  color: Colors.black,
                  border: Border.all(
                      color: Colors.grey.shade700
                  ),
                ),
                child: RotatedBox(
                  quarterTurns: -3, //Rotate the TabBar for vertical look
                  child: TabBar(
                    tabs: [
                      Tab(child: RotatedBox(quarterTurns: -1, child: Text('Personalization', maxLines: 1, softWrap: false,))),
                      Tab(child: RotatedBox(quarterTurns: -1, child: Text('Drone Information', maxLines: 1,))),
                      Tab(child: RotatedBox(quarterTurns: -1, child: Text('Gesture Settings', maxLines: 2, overflow: TextOverflow.visible,))),
                      Tab(child: RotatedBox(quarterTurns: -1, child: Text('Flight logs', maxLines: 2,))),
                    ],
                    labelColor: Colors.white,
                    unselectedLabelColor: Colors.grey,
                    indicator: BoxDecoration(
                      color: Colors.grey.shade700,
                      shape: BoxShape.rectangle,
                    ),
                    indicatorColor: Colors.grey.shade700,
                  ),
                ),
              ),
              Expanded(
                child: TabBarView(
                  children: [
                    //Drone info tab and down in order of tab creation
                    PersonalizationPage(),
                    DroneInfoPage(info: info,),
                    GestureControlPage(),
                    FlightLogsPage()
                  ],
                ),
              ),
            ],
          ),
        )
    );
  }
}