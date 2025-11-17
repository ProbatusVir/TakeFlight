import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:ffmpeg_kit_flutter_new/ffmpeg_kit.dart';
import 'package:image/image.dart' as img;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';

GlobalKey previewKey = GlobalKey();
List<Uint8List> capture = [];
late Timer recordingTimer;

Future<void> captureFrames() async{
  final boundary = previewKey.currentContext!.findRenderObject()
      as RenderRepaintBoundary;
  //Capture Raw Image
  final uiImage = await boundary.toImage(pixelRatio: 1.0);

  //convert to raw RGBA byte buffer
  final byteData = await uiImage.toByteData(format: ui.ImageByteFormat.rawRgba);

  if (byteData == null){
    throw Exception('Could not get byte data from ui.Image');
  }

  final rgbaBytes = byteData.buffer;

  // Create a package:image.Image from the raw RGBA bytes
  final img.Image image = img.Image.fromBytes(
    width: uiImage.width,
    height: uiImage.height,
    bytes: rgbaBytes,
    format: img.Format.uint8,
  );

  //encode image to Jpeg
  final Uint8List jpegBytes = img.encodeJpg(image, quality: 85);
}

void startRecording(){
  capture.clear(); //clear list incase its still has data in it
  //start a timer
  recordingTimer = Timer.periodic(Duration(milliseconds: 33), (_){
    captureFrames();
  });
}

void stopRecording() async{
recordingTimer.cancel();
// save captured frames to mp4
await encodeToVideo(capture);

}

Future<void> encodeToVideo(List<Uint8List> capture) async {
 final dir = Directory.current.path.replaceAll("\\", "/");
 final recordingDir = Directory('$dir/assets/Recordings');

 final output = '${recordingDir.path}/testRec.mp4';

 //using ffmpeg to build video at 30fps
  final command = '''
  ffmpeg -f rawvideo -pix_fmt rgba -s WIDTHxHEIGHT -i - $output.mp4
  ''';

  //Execute command
  await FFmpegKit.execute(command);
}

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
          stopRecording();
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