import 'package:flutter/material.dart';

class Settings extends StatelessWidget{
  const Settings({super.key});

  @override
  Widget build(BuildContext context){
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      home: DefaultTabController(
          length: 4,
          child: Scaffold(
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
                  ),
                  child: RotatedBox(
                      quarterTurns: -3, //Rotate the TabBar for vertical look
                    child: TabBar(
                        tabs: [
                          Tab(child: RotatedBox(quarterTurns: -1, child: Text('Drone Information', maxLines: 1,))),
                          Tab(child: RotatedBox(quarterTurns: -1, child: Text('Settings', maxLines: 1,))),
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
                          Container(
                            color: Colors.grey.shade700,
                            child: Text('Drone1:',
                              style: TextStyle(
                                fontWeight: FontWeight.bold,
                                fontSize: 30.0,
                              ),
                            ),
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