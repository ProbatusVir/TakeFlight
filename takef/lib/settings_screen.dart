import 'package:flutter/material.dart';

class Settings extends StatelessWidget{
  const Settings({super.key});

  @override
  Widget build(BuildContext context){
    return MaterialApp(
      home: DefaultTabController(
          length: 4,
          child: Scaffold(
            body: Row(
              children: [
                //Left side tab bar
                Container(
                  width: 80,
                  height: 80,
                  color: Colors.black,
                  child: RotatedBox(
                      quarterTurns: 3, //Rotate the TabBar for vertical look
                    child: TabBar(
                        tabs: [
                          Tab(text: 'Drone Information'),
                          Tab(text: 'Settings'),
                          Tab(text: 'Gesture Settings'),
                          Tab(text: 'Flight logs'),
                        ],
                      labelColor: Colors.white,
                      unselectedLabelColor: Colors.black,
                      indicatorColor: Colors.grey.shade700,
                    ),
                  ),
                ),
                Expanded(
                    child: TabBarView(
                        children: [
                          //Drone info tab and down in order of tab creation
                          Center(
                            child: Text('Drone info content'),
                          ),
                          Center(
                            child: Text('Settings content'),
                          ),
                          Center(
                            child: Text('Gesture Settings content'),
                          ),
                          Center(
                            child: Text('Flight Logs'),
                          ),
                        ],
                    ),
                ),
              ],
            ),
          )
      )
    );
  }
}