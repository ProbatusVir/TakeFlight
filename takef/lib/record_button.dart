import 'dart:async';
import 'dart:io';
import 'package:ffmpeg_kit_flutter_new/ffmpeg_kit.dart';
import 'package:image/image.dart' as img;
import 'package:path/path.dart' as path;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';


class RecordButton extends StatefulWidget{
  const RecordButton({super.key, required this.getFrames});
  final List<Uint8List>? Function() getFrames;

  @override
  State<RecordButton> createState() => _RecordButtonState();
}

class _RecordButtonState extends State<RecordButton>{
  bool isRecording = false;
  Timer? capture;
  late String outPath;
  late String pngPath;
  late Directory frameDir;

  void startRecording() async{
    //create output path
    final dir = Directory.current.path;
    final timeStamp = DateTime.now().millisecondsSinceEpoch;
    outPath = path.join(dir, 'assets', 'Recordings', 'testRec-$timeStamp.mp4');
    //setup file directory to place a png list
    final pngDir = path.join(dir, 'assets', 'Recordings', 'PngFrames');
    pngPath = path.join(pngDir, 'frames-$timeStamp');
    frameDir = Directory(pngPath);
    //creates the directory in case it doesn't exist
    if(!await frameDir.exists()){
      await frameDir.create(recursive: true);
    }
    
    int frameCount = 0; //count for the number of frames
    //capture frames at 20 fps
    capture = Timer.periodic(Duration(milliseconds: 50), (timer) async {
      //get images from video_feed file
      final jpeg = widget.getFrames();
      if(jpeg == null || jpeg.isEmpty) throw Exception('Error: No received jpeg Images');
      //loop through to get all the bytes
      for(final frames in jpeg){
        //decode images to turn into png images
        final decode = img.decodeJpg(frames);
        if(decode == null) {
          print('Warning: Skipped a frame due to decode failure');
          continue;
        }
        final framePath = path.join(pngPath, 'frame_${frameCount.toString().padLeft(4, '0')}.png');
        await File(framePath).writeAsBytes(img.encodePng(decode));
        frameCount++;
      }
    });
  }

  void stopRecording() async{
    //stops timer
    capture?.cancel();
    //Finishes the video encoder and saves it
   final command = '-framerate 20 -i $frameDir/frame_%04d.png '
       '-c:v libx264 -pix_fmt yuv420p $outPath';
   await FFmpegKit.execute(command);
  }

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