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
                          Tab(child: RotatedBox(quarterTurns: -1, child: Text('Drone Information', maxLines: 1,))),
                          Tab(child: RotatedBox(quarterTurns: -1, child: Text('Settings', maxLines: 1, softWrap: false,))),
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
                          Stack( // Drone info Tab
                            children: [
                              Align(
                                alignment: Alignment.topLeft,
                                child: Column(
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
                              Align(
                                alignment: Alignment.topRight,
                                child: BackButton(
                                  color: Colors.white,
                                  onPressed: (){
                                    Navigator.of(context).pop();
                                  },
                                ),
                              ),
                              Align(
                                alignment: Alignment.center,
                                child: Column(
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