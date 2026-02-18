import 'dart:async';
import 'dart:io';
import 'dart:convert'; //For encoding/decoding
import 'dart:math';
import 'main.dart';
import 'package:flutter/cupertino.dart' hide ConnectionState;
import 'package:flutter/foundation.dart'; //For Uint8List
import 'central_screen.dart';
import 'video_feed.dart';
class ControlRC{
  Socket? controlSoc;

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

  void sendLanding(int landC){
    if(controlSoc == null){
      print('socket not ready');
      return;
    }
    final landPac = [
      0x01, //landing command
      landC //Landing code
    ];
    print('Sending landing packet');
    controlSoc?.add(landPac);
  }

  void sendTakeOff(){
    if(controlSoc == null){
      print('socket not ready');
      return;
    }
    final takePac = [
      0x00, //take off command
      0x00, //reserved
    ];

    print('Sending Take off packet');
    controlSoc?.add(takePac);
  }

  Future<void> connect(int handshake, int port) async{
    try{
      controlSoc = await Socket.connect('127.0.0.1', port);
      print('Connected to Server over Control Socket: ${controlSoc?.remoteAddress}:${controlSoc?.remotePort}');
    } on SocketException catch (e){
      print("Error connecting to server on Control: $e");
    }
    //Send server handshake
    if(controlSoc != null){
      controlSoc?.add([0x42, 0x42, handshake]);
    }
  }

}

class Info{
  Socket? infoSoc;
  final List<int> dataBuffer = [];

  //Completers for awaiting responses
  Completer<ConnectionState>? connectionCompleter;
  Completer<List<String>>? ssidCompleter;
  Completer<Map<String, dynamic>>? infoDump;

  Future<void> connect (int port) async{
    try{
      infoSoc = await Socket.connect('127.0.0.1', port);
      //Prints are for debugging
      print('Connected to Server over Info Socket: ${infoSoc?.remoteAddress}:${infoSoc?.remotePort}');
    }on SocketException catch (e){
      print("Error connecting to server on Info: $e");
    }
    if(infoSoc != null){
      //send handshake
      infoSoc?.add([0x42, 0x42, 0x03]);
      await infoSoc?.flush();
      print('Info Handshake was sent');
    }

    if(infoSoc != null){
      infoSoc?.listen(
        (Uint8List data){
          print("Received ${data.length} bytes from server: ${data.toList()}");
          dataBuffer.addAll(data);
          processBufferData();
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

  Future<void> infoID(int infoID) async{
    if(infoSoc != null){
      //send data [INFO_ID : u8, RO_SHAM_BO : u8, payload_size : u16, PAYLOAD]
      final packet = Uint8List.fromList([
        infoID,  //SSID
        0x01,  //RoShamBo
        0x00, 0x00, //payload_size
      ]);
      infoSoc?.add(packet);
      //one flush
      await infoSoc?.flush();
      print('Info $infoID packet sent');
    }
  }

  /*void socketData(Uint8List data){
    dataBuffer.addAll(data);
    processBufferData();
  }*/

  void processBufferData(){
    //loop through buffer and get first 4 bytes(header)
    print("RAW HEADER BYTES: ${dataBuffer.sublist(0, 4)}");
    while(true){
      if (dataBuffer.length < 4) return;

      final id = dataBuffer[0];

      final roShamBo = dataBuffer[1];

      final payloadSize = dataBuffer[2];

      final fullPacket = 4 + payloadSize;

      //wait for entire packet
      if(dataBuffer.length < fullPacket) return;

      final packet = Uint8List.fromList(dataBuffer.sublist(0,fullPacket));

      dataBuffer.removeRange(0, fullPacket);

      handleData(id, roShamBo, packet.sublist(4));
    }
  }

  void handleData(int id, int roShamBo, Uint8List payload){
    //print("Received info data: $data");
    //final int type = data[0];

    switch(id){
      ///SSIDS
      case 0x00:
        handleSSIDList(payload);
        break;
        ///DroneStateDump
      case 0x01:
        handleDroneDump(payload);
        break;
        ///Record Request
      case 0x02:
        break;
        ///DroneConnection
      case 0x03:
        handleConnectionState(payload);
        break;
        ///Drone Selection
      case 0x04:
        break;
    }
  }

  void handleSSIDList(Uint8List data) {
    //receive SSID
    List<String> recSSID = [];
    try {
      //decode received data
      final recData = utf8.decode(data, allowMalformed: true);
      print('Received: $recData');
      //decode json
      final jString = recData.substring(recData.indexOf('{'));
      final decoded = jsonDecode(jString); // decoded is a Map<String, dynamic>
      recSSID = List<String>.from(decoded['ssids']);
      print('The received SSIDs: $recSSID');
      ssidCompleter!.complete(recSSID);
      ssidCompleter = null;
    } catch (e) {
      ssidCompleter!.completeError(e);
      ssidCompleter = null;
    }
  }

  void handleDroneDump(Uint8List data){
    Map<String,dynamic> droneInfo;

    try{
      final payload = data.sublist(1);

      //6 bytes length for ssid error
      final ssidBytes = payload.sublist(0, 6);
      final isInvalid = ssidBytes.every((b) => b == 0x00);

      //check to see if its invalid
      if(isInvalid){
        infoDump!.completeError(
          StateError("Drone Dump unavailable")
        );
        infoDump = null;
        return;
      }

      //json data
      final jBytes = payload.sublist(5);
      final jMap = utf8.decode(jBytes);
      final decode = jsonDecode(jMap);
      if(decode == null){
        debugPrint("There is no drone info available");
        return;
      }
      droneInfo = decode;
      infoDump!.complete(droneInfo);
      infoDump = null;
    } catch (e){
      infoDump!.completeError(e);
      infoDump = null;
    }
  }

  void handleConnectionState(Uint8List data){
    //print("Received Connection State data: $data");
    final int code = data.length > 1 ? data[1] : 255; //assuming the received connection state is index 1
    final state = ConnectionState.fromCode(code);
    connectionCompleter!.complete(state);
    connectionCompleter = null;
  }

  Future<List<String>> receiveSSID() async{
    ssidCompleter = Completer<List<String>>();
    return ssidCompleter!.future;
  }

  Future<Map<String, dynamic>> recieveDroneInfo() async{
    infoDump = Completer<Map<String, dynamic>>();
    return infoDump!.future;
  }

  Future<ConnectionState> connection() async{
    connectionCompleter = Completer<ConnectionState>();
    return connectionCompleter!.future;
  }
  void sendSSID(String ssid) async{
    final ssidByte = utf8.encode(ssid);
    if(infoSoc != null){
      infoSoc?.add(ssidByte);
      print("Sent selected SSID");
    }
  }
}

class DroneVideo{
  Socket? videoSoc;

  Future<void> connect(int port, GlobalKey<VideoFeedState> videoKey) async {
    try {
      videoSoc = await Socket.connect('127.0.0.1', port);
      //Prints are for debugging
      print('Connected to Server over video socket: ${videoSoc
          ?.remoteAddress}:${videoSoc?.remotePort}');
    } on SocketException catch (e) {
      print("Error connecting to server on Video: $e");
    }

    //send data to server
    if(videoSoc != null){
      videoSoc?.add([0x42, 0x42, 0x02]); //sends the header bytes along with the ID of video stream
      //socket.flush(); //ensures all data is sent
    }
    List<int> imageDataBytes = [];
    if(videoSoc != null) {
      videoSoc?.listen(
              (Uint8List data) {
            //print('Received chunk of ${data.length} bytes: ${data.take(16).toList()}');
            imageDataBytes.addAll(data);
            //loop through to find the start of the png then reassemble
            while (true) {
              int sigIndex = findStart(imageDataBytes);
              if (sigIndex == -1) break;

              //Look for PNG end marker
              int endIndex = findEnd(imageDataBytes, sigIndex);
              if (endIndex == -1) break; //not complete

              //Extract full PNG
              Uint8List pngBytes = Uint8List.fromList(
                  imageDataBytes.sublist(sigIndex, endIndex));
              print('Successfully found png in connect file');
              videoKey.currentState?.onImageReceived(pngBytes);

              //Remove consumed bytes
              imageDataBytes.removeRange(0, endIndex + 8);
            }
          }
      );
    }
  }

  Future<void> getDroneImg(GlobalKey<VideoFeedState> videoKey) async{
    List<int> imageDataBytes = [];
    if(videoSoc != null) {
      videoSoc?.listen(
              (Uint8List data) {
            //print('Received chunk of ${data.length} bytes: ${data.take(16).toList()}');
            imageDataBytes.addAll(data);
            //loop through to find the start of the png then reassemble
            while (true) {
              int sigIndex = findStart(imageDataBytes);
              if (sigIndex == -1) break;

              //Look for PNG end marker
              int endIndex = findEnd(imageDataBytes, sigIndex);
              if (endIndex == -1) break; //not complete

              //Extract full PNG
              Uint8List pngBytes = Uint8List.fromList(
                  imageDataBytes.sublist(sigIndex, endIndex));
              print('Successfully found png in connect file');
              videoKey.currentState?.onImageReceived(pngBytes);

              //Remove consumed bytes
              imageDataBytes.removeRange(0, endIndex + 8);
            }
          }
      );
    }
    /*final completer = Completer<Uint8List?>();
    //receiving image data
    if(videoSoc != null){
      videoSoc?.listen(
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
              final pngBytes = Uint8List.fromList(imageDataBytes.sublist(sigIndex, endIndex));
              print('Successfully found png in connect file');


              //Remove consumed bytes
              imageDataBytes.removeRange(0, endIndex+8);
              if(!completer.isCompleted){
                completer.complete(pngBytes);
              }
            }
          },
          onDone: (){
            print('Server disconnected');
            videoSoc?.destroy();
            if (!completer.isCompleted) completer.complete(null);
          },
          onError: (error){
            print('Error on socket: $error');
            videoSoc?.destroy();
            if (!completer.isCompleted) completer.complete(null);
          }
      );
    }
    return completer.future;*/
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
}

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
  final fileExt = Platform.isWindows ? '.exe' : '';
  final process = Process.start(
    "$normalPath/target/$targetMode/TakeFlight$fileExt",
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