import 'dart:async';
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';

GlobalKey previewKey = GlobalKey();
List<Uint8List> capture = [];
late Timer recordingTimer;

Future<void> captureFrames() async{
  RenderRepaintBoundary boundary =
      previewKey.currentContext!.findRenderObject() as RenderRepaintBoundary;
  var image = await boundary.toImage(pixelRatio: 1.0);
  ByteData? byteData = (await image.toByteData(format: ImageByteFormat.png));
  if(byteData != null){
    capture.add(byteData.buffer.asUint8List());
  }
}

void startRecording(){
  /*capture.clear(); //clear list incase its still has data in it
  //start a timer
  recordingTimer = Timer.periodic(Duration(milliseconds: 20), (timer){
    captureFrames();
  });*/
  throw UnimplementedError();
}

void stopRecording() async{
/*recordingTimer.cancel();
// save captured frames to mp4
await encodeToVideo(capture);*/
throw UnimplementedError();
}

/*Future<void> encodeToVideo(List<Uint8List> capture) async {
 final dir = await getTemporaryDirectory();
 for (int i = 0; i < capture.length; i++){
   final file = File()
 }
}*/

class RecordButton extends StatefulWidget{
  const RecordButton({super.key});
  @override
  State<RecordButton> createState() => _RecordButtonState();
}

class _RecordButtonState extends State<RecordButton>{
  bool isRecording = false;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: (){
        setState((){
          isRecording = !isRecording; //simpler way to set it to true or false
        });
        if(isRecording){
          //function to start recording
          startRecording();
        }else{
          //function to stop recording
          startRecording();
        }
      },
      child: isRecording? SvgPicture.asset(
        'assets/Images/Stop_Circle.svg',
        width: 50,
        height: 50,
        semanticsLabel: 'Stop Recording',
      )
          :SvgPicture.asset(
        'assets/Images/record_icon.svg',
        width: 50,
        height: 50,
        semanticsLabel: 'Record',
      ),
    );
  }
}