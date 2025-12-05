import 'dart:ffi';
import 'dart:io';
import 'dart:convert'; //For encoding/decoding
import 'dart:math';
import 'package:flutter/foundation.dart'; //For Uint8List
import 'package:takef/main.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

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



Future<void> androidConnect()async{
  /*final debug = File("helloworld.txt");
  await debug.writeAsString("${ await getServerPort() }");*/

  Socket? socket;
  int port = await getServerPort();
  try{
    socket = await Socket.connect('127.0.0.1', port);
    //Prints are for debugging
    print('Connected to Server: ${socket.remoteAddress}:${socket.remotePort}');
  }on SocketException catch (e){
    print("Error connecting to server: $e");
  }
  //func for getting images from drone
  await getDroneImg(socket, port);
  //func for getting SSID from server
  await getSSID(socket, port);
}

Future<void> getDroneImg(Socket? socket, int port) async {
  //send data to server
  if(socket != null){
    socket.add([0x42, 0x42, 0x02]); //sends the header bytes along with the ID of video stream
    //socket.flush(); //ensures all data is sent
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
          socket.destroy();
        },
        onError: (error){
          print('Error on socket: $error');
          socket.destroy();
        }
    );
  }
}

Future<void> getSSID(Socket? socket, int port) async {
  try{
    socket = await Socket.connect('127.0.0.1', port);
    //Prints are for debugging
    print('Connected to Server for SSID: ${socket.remoteAddress}:${socket.remotePort}');
  }on SocketException catch (e){
    print("Error connecting to server: $e");
  }
  if(socket != null){
    //send handshake
     socket.add([0x42, 0x42, 0x03]);
     print('Info Handshake was sent');
     //send data [INFO_ID : u8, RO_SHAM_BO : u8, payload_size : u16, PAYLOAD]
     final packet = Uint8List.fromList([
       0x00,
       0x01,
       0x00, 0x04,
       0xAA, 0xBB, 0xCC, 0xDD
     ]);
     socket.add(packet);
     //one flush
     //socket.flush();
  }
  //receive SSID
  List<String> recSSID = [];
  if(socket != null){
    socket.listen(
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
        socket?.destroy();
      },
      onDone: (){
        print('Server disconnected');
        socket?.destroy();
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

Future<void> connectToServer() async{
  //Need to detect platform first to determine correct socket creation
  if(kIsWeb){
    //await webConnection();
  }else{
    await androidConnect();
  }

}