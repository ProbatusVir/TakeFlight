import 'dart:io';
import 'dart:convert'; //For encoding/decoding
import 'dart:typed_data'; //For Uint8List
import 'package:flutter/material.dart';
//import 'package:flutter_svg/flutter_svg.dart'; //svg package handler
import 'flight_screen.dart';

Future<void> connectToServer() async{
  //Need to detect platform first to determine correct socket creation
  Socket? socket;
  try{
    socket = await Socket.connect('192.168.1.137', 57585);
    //Prints are for debugging
    print('Connected to Server: ${socket.remoteAddress}:${socket.remotePort}');
  }on SocketException catch (e){
    print("Error connecting to server: $e");
  }
  //send data to server
  if(socket != null){
    socket.add([0x42, 0x42, 2]); //sends the header bytes along with the ID of video stream
    socket.flush(); //ensures all data is sent
  }
  List<int> imageDataBytes = [];
  int? imageLength;
  //receiving image data
  if(socket != null){
    socket.listen(
            (Uint8List data){
          if(imageLength == null && data.length >= 4){ //Assuming 4 bytes for length
            imageLength = ByteData.view(data.buffer).getInt32(0, Endian.big); //Read length
            imageDataBytes.addAll(data.sublist(4)); //read the rest of data after first 4
          } else if(imageLength != null){
            imageDataBytes.addAll(data);
          }
          if(imageLength != null && imageDataBytes.length >= imageLength!){
            //image data fully received
            Uint8List receivedImage = Uint8List.fromList(imageDataBytes.sublist(0, imageLength!));
            print('Image received with ${receivedImage.length} bytes.');

            // Can now use this function (e.g., display it using Image.memory)
            // Reset for next image if multiple images are expected
            imageDataBytes.clear();
            imageLength = null;
          }
        },
        onDone: (){
          print('Server disconnected');
          socket?.destroy();
        },
        onError: (error){
          print('Error on socket: $error');
          socket?.destroy();
        }
    );
  }
}

void main() async {
  WidgetsFlutterBinding.ensureInitialized(); //ensures flutter is initialized
  //Connect to rust server
  await connectToServer();
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
        scaffoldBackgroundColor: Colors.black,
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
//creates list this will later be the get call for drone names
  final List<String> items = List.generate(3, (index) => 'Drone ${index + 1}');

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      //TODO::Change out logo text with SVG text fix logo size for mobile
      backgroundColor: Colors.black,//changes the overall scaffold color which is the background of the screen itself
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
            Image.asset('assets/Images/drone_icon.png'),
            Text(
              'TAKEFLIGHT',
              style: Theme.of(context).textTheme.displayLarge,
            ),
            FloatingActionButton.extended( //extends the button to fit its contents
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
                                  return ListTile(
                                    title: Text(items[index]),
                                    trailing: Icon(Icons.wifi_outlined, color: Colors.white),
                                    textColor: Colors.white,
                                    onTap: (){
                                      //notifies user they connected
                                      ScaffoldMessenger.of(context).showSnackBar(
                                          SnackBar(content: Text('Connecting to...Drone${index +1}'))
                                      );
                                      //goes to main screen after connection
                                      Navigator.of(context).push(
                                          MaterialPageRoute(builder: (BuildContext context) => FlightScreen())
                                      );
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