import 'dart:ffi';
import 'dart:io';
import 'dart:convert'; //For encoding/decoding
import 'dart:math';
import 'package:flutter/cupertino.dart';
import 'package:flutter/foundation.dart'; //For Uint8List
import 'package:takef/main.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'flight_screen.dart';
import 'video_feed.dart';

Socket? controlSoc;

/// This method may throw IOException or RangeError.
Future<int> getServerPort() async {
  // Acquire random socket
  final sock = await RawDatagramSocket.bind(
    InternetAddress.anyIPv4, // Trust me, IPv4
    0,
  );

  // Start the server process using relative paths.
  // FIXME: Maybe you'll want a ternary operator to decide whether to use Debug or Release in the path.
  final normalPath = Directory.current.parent.path.replaceAll("\\", "/");
  final targetMode = (kDebugMode ? 'debug': 'release');
  final process = Process.start(
    "$normalPath/target/$targetMode/TakeFlight.exe",
    [sock.port.toString(),],
    mode: ProcessStartMode.inheritStdio,
    workingDirectory: "..",
  );


  // This is a blocking read until we get a readable message. The first -- and only -- message we receive should be a u16, though additional error handling can be implemented here.
  int? serverPort;
  await for (RawSocketEvent event in sock) {
    if (event == RawSocketEvent.read) {
      final message = sock.receive();
      if (message != null) {
        serverPort = ByteData.view(message.data.buffer).getUint16(0, Endian.big);
        break;
      }
    }
  }

  // Sanitize and validate server port number. As a side note: I wish Dart had a constant for unsigned 16-bit max
  if (serverPort == null) { throw Error.throwWithStackTrace(IOException, StackTrace.current); }
  if (serverPort < 0 || serverPort > pow(2, 16) - 1) { throw Error.throwWithStackTrace(RangeError.value(serverPort, "serverPort", "The received port number was out of range of any OS port."), StackTrace.current); }
  return Future.value(serverPort);
}



Future<void> androidConnect(GlobalKey<VideoFeedState> videoKey)async{
  /*final debug = File("helloworld.txt");
  await debug.writeAsString("${ await getServerPort() }");*/

  int port = await getServerPort();
  //func for control handshake
  //await controlRC(port);
  //func for getting images from drone
  await getDroneImg(port, videoKey);
  //func for getting SSID from server
  //await getSSID(port);
}

void sendRC(){
  if(controlSoc == null){
    print('socket not ready');
    return;
  }
  if(rcCon.packet.isEmpty){
    print('Packet is currently empty');
    return;
  }
  print('Sending movement packet...');
  controlSoc?.add(rcCon.packet);
}

Future<void> controlRC(int port) async{
  try{
    controlSoc = await Socket.connect('127.0.0.1', port);
    print('Connected to Server over Control Socket: ${controlSoc?.remoteAddress}:${controlSoc?.remotePort}');
  } on SocketException catch (e){
    print("Error connecting to server on Control: $e");
  }
  //Send server handshake
  if(controlSoc != null){
    controlSoc?.add([0x42, 0x42, 0x01]);
  }
}

Future<void> getDroneImg(int port, GlobalKey<VideoFeedState> videoKey) async {
  Socket? videoSoc;
  try{
    videoSoc = await Socket.connect('127.0.0.1', port);
    //Prints are for debugging
    print('Connected to Server over video socket: ${videoSoc.remoteAddress}:${videoSoc.remotePort}');
  }on SocketException catch (e){
    print("Error connecting to server on Video: $e");
  }
  //send data to server
  if(videoSoc != null){
    videoSoc.add([0x42, 0x42, 0x02]); //sends the header bytes along with the ID of video stream
    //socket.flush(); //ensures all data is sent
  }
  List<int> imageDataBytes = [];
  int? imageLength;
  //receiving image data
  if(videoSoc != null){
    videoSoc.listen(
            (Uint8List data){
              //print('Received chunk of ${data.length} bytes: ${data.take(16).toList()}');
              imageDataBytes.addAll(data);
              //loop through to find the start of the png then reassemble
              while(true) {
                int sigIndex = findStart(imageDataBytes);
                if(sigIndex == -1) break;

                //Look for PNG end marker
                int endIndex = findEnd(imageDataBytes, sigIndex);
                if(endIndex == -1)break; //not complete

                //Extract full PNG
                Uint8List pngBytes = Uint8List.fromList(imageDataBytes.sublist(sigIndex, endIndex));
                print('Successfully found png in connect file');
                videoKey.currentState?.onImageReceived(pngBytes);

                //Remove consumed bytes
                imageDataBytes.removeRange(0, endIndex+8);
              }
          /*if(imageLength == null && data.length >= 4){ //Assuming 4 bytes for length
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
          }*/
        },
        onDone: (){
          print('Server disconnected');
          videoSoc?.destroy();
        },
        onError: (error){
          print('Error on socket: $error');
          videoSoc?.destroy();
        }
    );
  }
}

int findStart(List<int> buf) {
  for (int i = 0; i < buf.length - 3; i++) {
    if (buf[i] == 137 && buf[i+1] == 80 && buf[i+2] == 78 && buf[i+3] == 71) {
      return i;
    }
  }
  return -1;
}


int findEnd(List<int> buf, int start) {
  for (int i = start; i < buf.length-7; i++) {
    if (buf[i] == 73 && buf[i+1] == 69 && buf[i+2] == 78 && buf[i+3] == 68 &&
        buf[i+4] == 174 && buf[i+5] == 66 && buf[i+6] == 96 && buf[i+7] == 130) {
      return i;
    }
  }
  return -1;
}


Future<void> getSSID(int port) async {
  Socket? infoSoc;
  try{
    infoSoc = await Socket.connect('127.0.0.1', port);
    //Prints are for debugging
    print('Connected to Server over Info Socket: ${infoSoc.remoteAddress}:${infoSoc.remotePort}');
  }on SocketException catch (e){
    print("Error connecting to server on Info: $e");
  }
  if(infoSoc != null){
    //send handshake
     infoSoc.add([0x42, 0x42, 0x03]);
     print('Info Handshake was sent');
     //send data [INFO_ID : u8, RO_SHAM_BO : u8, payload_size : u16, PAYLOAD]
     final packet = Uint8List.fromList([
       0x00,  //SSID
       0x01,  //RoShamBo
       0x00, 0x00, //payload_size
     ]);
     infoSoc.add(packet);
     //one flush
     await infoSoc.flush();
  }
  //receive SSID
  List<String> recSSID = [];
  if(infoSoc != null){
    infoSoc.listen(
      (Uint8List data){
        //decode received data
        final recData = utf8.decode(data, allowMalformed: true);
        print('Received: $recData');
        //decode json
        final jString = recData.substring(recData.indexOf('{'));
        final decoded = jsonDecode(jString); // decoded is a Map<String, dynamic>
        recSSID = List<String>.from(decoded['ssids']);
        print('The received SSIDs: $recSSID');
      },
      onError: (e){
        print('Error on socket: $e');
        infoSoc?.destroy();
      },
      onDone: (){
        print('Server disconnected');
        infoSoc?.destroy();
      }
    );
  }
}

Future<void> webConnection() async{
  final port = 51108;
  final web = WebSocketChannel.connect(
      Uri.parse('ws://localhost:$port'));
  //Send data to server
  web.sink.add([0x42, 0x42, 2]);
}

Future<void> connectToServer(GlobalKey<VideoFeedState> videoKey) async{
  //Need to detect platform first to determine correct socket creation
  if(kIsWeb){
    //await webConnection();
  }else{
    await androidConnect(videoKey);
  }

}