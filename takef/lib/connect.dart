import 'dart:io';
import 'dart:convert'; //For encoding/decoding
import 'package:flutter/foundation.dart'; //For Uint8List
import 'package:web_socket_channel/web_socket_channel.dart';



Future<void> androidConnect()async{
  Socket? socket;
  try{
    socket = await Socket.connect('10.0.0.215', 51108);
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
    await webConnection();
  }else{
    await androidConnect();
  }

}