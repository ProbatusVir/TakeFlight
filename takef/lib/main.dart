import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:takef/personalization_tab.dart';

import 'connect.dart';
import 'package:flutter/material.dart';
//import 'package:flutter_svg/flutter_svg.dart'; //svg package handler
import 'central_screen.dart';
import 'drone_info_tab.dart';
import 'flight_logs_tab.dart';
import 'gesture_tab.dart';
import 'settings_screen.dart';
import 'msettings_screen.dart';

void main() async {
  //WidgetsFlutterBinding.ensureInitialized(); //ensures flutter is initialized
  //Connect to rust server
  //await connectToServer();
  runApp(const MyApp());
}

final info = Info();
Map<String, dynamic> droneInfo = {};

enum ConnectionState{
  connecting(0),
  connected(1),
  failed(2),
  disconnected(3),
  unavailable(255);

  final int code;
  const ConnectionState(this.code);

  static ConnectionState fromCode(int code) {
    return ConnectionState.values.firstWhere(
          (state) => state.code == code,
      orElse: () => ConnectionState.unavailable,
    );
  }
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'TakeFlight',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
            seedColor: Colors.grey.shade700,
        ),
        //Color for text theme
        textTheme: TextTheme(
          displayLarge: TextStyle(
            //color: Colors.white,
            foreground: Paint()
              ..style = PaintingStyle.stroke //set the style to stroke
              ..strokeWidth = 2 //defines the width of the strok
              ..color = Colors.white, //set the stroke color
          ),
          headlineMedium: TextStyle(color: Colors.black), //raw hex value til style file is created
          headlineLarge: TextStyle(color: Colors.white),
          bodyLarge: TextStyle(color: Colors.white),
          bodyMedium: TextStyle(color: Colors.white),
        ),
        scaffoldBackgroundColor: Colors.black,
      ),
      debugShowCheckedModeBanner: false, //gets rid of debug sash
      home: const MyHomePage(title: 'TakeFlight'),
      routes: {
        '/home': (_) => const MyHomePage(title: 'TakeFlight'),
        '/personalization': (_) =>  const PersonalizationPage(),
        '/drone-info': (_) => DroneInfoPage(info: droneInfo,),
        '/gesture-control': (_) => const GestureControlPage(),
        '/flight-logs': (_) => const FlightLogsPage()
      },
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});
  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
//creates list this will later be the get call for drone names
  final List<String> fakeItems = List.generate(3, (index) => 'Drone ${index + 1}');
  List<String> items = [];
  int port = 0;

  @override
  void initState(){
    super.initState();
    startInfo();
  }

  void startInfo() async{
    port = await getServerPort();
    await info.connect(port);
    await info.infoID(0x00); ///SSID
    items = await info.receiveSSID();
  }

  @override
  Widget build(BuildContext context) {
    if(items.isEmpty){
      items = fakeItems;
    }
    return Scaffold(
      //TODO::Change out logo text with SVG text fix logo size for mobile
      backgroundColor: Colors.black,//changes the overall scaffold color which is the background of the screen itself
      // settings button
      floatingActionButton: FloatingActionButton(
        backgroundColor: Colors.black45,
          tooltip: "Settings",
          onPressed: (){
          if(defaultTargetPlatform == TargetPlatform.android){
            Navigator.of(context).push(
                MaterialPageRoute(builder: (BuildContext context) => MsettingsScreen())
            );
          }else{
            Navigator.of(context).push(
                MaterialPageRoute(builder: (BuildContext context) => Settings(info: droneInfo,))
            );
          }
          },
          child: Icon(
            Icons.settings_outlined,
            color: Colors.white,
          ),
      ),
      floatingActionButtonLocation: FloatingActionButtonLocation.endTop, //places the setting button to the top right
      body: Center(
        // Center is a layout widget. It takes a single child and positions it
        // in the middle of the parent.
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          //returns multiple child widgets to place in the center
          children: <Widget>[
            //Logo image
            Image.asset('assets/Images/drone_icon.png'),
            Text(
              'TAKEFLIGHT',
              style: Theme.of(context).textTheme.displayLarge,
            ),
            FloatingActionButton.extended( //extends the button to fit its contents
              heroTag: 'ConnectionFAB',
              backgroundColor: Colors.grey.shade400, //grey with a shade value of 400 that gives the creamy look
                onPressed: (){
                //pop up for drone connection list
                  showDialog(
                      context: context,
                      builder: (BuildContext context){
                        return SimpleDialog(
                          title: const Text('Select Drone'),
                          children: [
                            Container( //using container instead of sized box for more options
                              width: 280,
                              height: 280,
                              decoration: BoxDecoration(
                                color: Colors.black,
                                borderRadius: BorderRadius.circular(8),
                              ),
                              child: ListView.separated(//allows the creation of a list with seperators
                                itemCount: items.length,
                                itemBuilder: (context, index){
                                  final String ssid = items[index];
                                  return ListTile(
                                    title: Text(ssid),
                                    trailing: Icon(Icons.wifi_outlined, color: Colors.white),
                                    textColor: Colors.white,
                                    onTap: () async{
                                      ScaffoldMessenger.of(context).showSnackBar(
                                          SnackBar(content: Text('Connecting to...$ssid'))
                                      );
                                      await info.infoID(0x04);
                                      info.sendSSID(ssid);
                                      //await info.infoID(0x01);
                                      try {
                                        droneInfo =
                                        await info.recieveDroneInfo().timeout(
                                            const Duration(seconds: 3));
                                      } on TimeoutException{
                                        debugPrint("Drone Info not available");
                                      }
                                      if(!mounted) return;
                                      //goes to main screen after connection
                                      Navigator.of(context).push(
                                          MaterialPageRoute(builder: (_) => FlightScreen(port: port, info: droneInfo))
                                      );
                                      /*await info.infoID(0x04);
                                      info.sendSSID(ssid);
                                      await info.infoID(0x03); ///DroneConnectionState
                                      final status = await info.connection();
                                      //do a mounted check to prevent crashes after await
                                      if(!mounted) return;
                                      switch (status){
                                        case ConnectionState.connecting:
                                          ScaffoldMessenger.of(context).showSnackBar(
                                              SnackBar(content: Text("Connecting to...drone-$ssid"))
                                          );
                                          break;
                                        case ConnectionState.connected:
                                          ScaffoldMessenger.of(context).showSnackBar(
                                            SnackBar(content: Text("Connected to drone-$ssid"))
                                          );
                                          try {
                                        droneInfo =
                                        await info.recieveDroneInfo().timeout(
                                            const Duration(seconds: 3));
                                      } on TimeoutException{
                                        debugPrint("Drone Info not available");
                                      }
                                          Navigator.push(
                                            context,
                                            MaterialPageRoute(builder: (_) => FlightScreen(port: port))
                                          );
                                          break;
                                        case ConnectionState.failed:
                                          ScaffoldMessenger.of(context).showSnackBar(
                                              SnackBar(content: Text("Failed to connect to drone-$ssid"))
                                          );
                                          //TODO::Implement reconnect menu/exit
                                          break;
                                        case ConnectionState.disconnected:
                                          ScaffoldMessenger.of(context).showSnackBar(
                                              SnackBar(content: Text("Disconnected from drone-$ssid"))
                                          );
                                          //TODO::Implement reconnect menu/exit
                                          break;
                                        case ConnectionState.unavailable:
                                          ScaffoldMessenger.of(context).showSnackBar(
                                              SnackBar(content: Text("Drone-$ssid is Unavailable"))
                                          );
                                          break;
                                      }*/
                                    },
                                  );
                                },
                                separatorBuilder: (BuildContext context, int index){
                                  return Divider(
                                    thickness: 2,
                                    color: Colors.white,
                                  );
                                },
                              ),
                            )
                          ],
                        );
                      }
                  );
                },
                label: Text(
                    'CONNECT',
                    style: Theme.of(context).textTheme.headlineMedium, //default text size and theme
                ),
            ),
          ],
        ),
      ),
      // This trailing comma makes auto-formatting nicer for build methods.
    );
  }
}