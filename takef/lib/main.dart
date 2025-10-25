import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart'; //svg package handler

void main() {
  runApp(const MyApp());
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
        ),
        scaffoldBackgroundColor: Colors.black45,
      ),
      debugShowCheckedModeBanner: false, //gets rid of debug sash
      home: const MyHomePage(title: 'TakeFlight'),
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      //TODO::Fix screen background color and change out logo text with svg text
      backgroundColor: Colors.black45,//changes the overall scaffold color which is the background of the screen itself
      // settings button
      floatingActionButton: FloatingActionButton(
        backgroundColor: Colors.black45,
          onPressed: (){},
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
            Image.asset('../Images/drone_icon.png'),
            Text(
              'TAKEFLIGHT',
              style: Theme.of(context).textTheme.displayLarge,
            ),
            FloatingActionButton.extended( //extends the button to fit its contents
              backgroundColor: Colors.grey.shade400, //grey with a shade value of 400 that gives the creamy look
                onPressed: (){},
                label: Text(
                    'CONNECT...',
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
