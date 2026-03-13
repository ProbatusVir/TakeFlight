import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
//import 'package:flutter_quick_video_encoder/flutter_quick_video_encoder.dart';
import 'package:image/image.dart' as img;
import 'package:path/path.dart' as path;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:path_provider/path_provider.dart';

//an isolate compute function has to be JSON safe
//will work with a void function just not ideal
void _encodeFrame(Map<String, dynamic> args){
  //using try catch to find the silent crashes of isolate
  try{
    final frames = args['image'];
    final fpath = args['path'];
    final index = args['index'];

    final decode = img.decodeJpg(frames);
    if (decode == null) throw Exception('Frame Skipped');

    final pngbytes = Uint8List.fromList(img.encodePng(decode));
    File('$fpath/frame_${index.toString().padLeft(4, '0')}.png')
        .writeAsBytesSync(pngbytes);
  }catch(e){
    //isolate crash
    print('Isolate error: $e');
  }
}

class RecordButton extends StatefulWidget{
  const RecordButton({super.key, required this.getFrames});
  final List<Uint8List>? Function() getFrames;

  @override
  State<RecordButton> createState() => _RecordButtonState();
}

class _RecordButtonState extends State<RecordButton>{
  bool isRecording = false;
  Timer? capture;
  Timer? _timer;
  Duration recordingDuration = Duration.zero;
  late String outPath;
  late String pngPath;
  late Directory frameDir;

  //separate isolate process to handle overloading of writing images to file
  Future<void> process(List<Uint8List> frames, String fpath) async {
   //loop here to avoid giving compute to much to do
    for(int i = 0; i < frames.length; i++){
      final result = await compute(_encodeFrame, {
        'image': frames[i],
        'path': fpath,
        'index': i
      }).catchError((e){
        debugPrint('Error: $e');
      });
    }
  }

  Future<void> startRecording() async{
    ///For recording timer label
    recordingDuration = Duration.zero;

    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      setState(() {
        recordingDuration += const Duration(seconds: 1);
      });
    });
    //create output path
    //final dir = Directory.current.path;
    final andDir =  await getApplicationDocumentsDirectory();
    final andPath = andDir.path;
    final timeStamp = DateTime.now().millisecondsSinceEpoch;
    //outPath = path.join(dir, 'assets', 'Recordings', 'testRec-$timeStamp.mp4');
    outPath = path.join(andPath, 'Recordings', 'testRec-$timeStamp.mp4');

    //setup file directory to place a png list
    //final pngDir = path.join(dir, 'assets', 'Recordings', 'PngFrames');
    //pngPath = path.join(pngDir, 'frames-$timeStamp');
    pngPath = path.join(andPath, 'Recordings', 'PngFrames', 'frames-$timeStamp');
    frameDir = Directory(pngPath);
    //creates the directory in case it doesn't exist
    if(!await frameDir.exists()){
      await frameDir.create(recursive: true);
    }

    /*FlutterQuickVideoEncoder.setup(
        width: 480,
        height: 840,
        fps: 20,
        videoBitrate: 1000000,
        profileLevel: ProfileLevel.any,
        audioChannels: 0,
        audioBitrate: 0,
        sampleRate: 0,
        filepath: outPath
    );*/

    //capture frames at 20 fps
    capture = Timer.periodic(Duration(milliseconds: 50), (timer) async {
      //get images from video_feed file
      final jpeg = widget.getFrames();
      if(jpeg == null || jpeg.isEmpty) throw Exception('Error: No received jpeg Images');
      //to lesson load during debug and looping before the process causing it to create thousands of isolates
      await process(jpeg, pngPath); //loop will now be done within the isolate
    });
    final pngFile = await frameDir.list().toList();
    int total = pngFile.length;
    debugPrint('$total');

  }

  Future<void> stopRecording() async{
    //Stops timer for recording duration
    _timer?.cancel();
    //stops timer
    capture?.cancel();
    //Finishes the video encoder and saves it
    //FlutterQuickVideoEncoder.finish();
   /*final command = '-framerate 20 -i $frameDir/frame_%04d.png '
       '-c:v libx264 -pix_fmt yuv420p $outPath';
   await FFmpegKit.executeAsync(command);*/
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () async {
        setState((){
          isRecording = !isRecording; //simpler way to set it to true or false
        });
        if(isRecording){
          //function to start recording
          await startRecording();
        }else{
          //function to stop recording
          await stopRecording();
        }
      },
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          AnimatedSwitcher(
              duration: const Duration(milliseconds: 300),
            child: isRecording
            ? Container(
              key: const ValueKey("recordingBadge"),
              padding: EdgeInsets.symmetric(
                horizontal: 12,
                  vertical: 6,
              ),
              margin: const EdgeInsets.only(bottom: 8),
              decoration: BoxDecoration(
                color: Colors.red,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(
                    Icons.fiber_manual_record_outlined,
                    color: Colors.white,
                    size: 14,
                  ),
                  const SizedBox(width: 5),
                  Text(
                    formatDuration(recordingDuration),
                    style: const TextStyle(
                      color: Colors.white,
                      fontWeight: FontWeight.bold,
                    ),
                  )
                ],
              ),
            )
                : const SizedBox.shrink(),
          ),
          Tooltip(
            message: isRecording ? "End Recording" : "Record",
            child: SvgPicture.asset(
              isRecording
                  ? 'assets/Images/Stop_Circle.svg'
                  : 'assets/Images/record_icon.svg',
              width: 50,
              height: 50,
              semanticsLabel:
              isRecording ? 'Stop Recording' : 'Record',
            ),
          ),
        ],
      )
    );
  }
}

///Helper
String formatDuration(Duration d){
  final minutes = d.inMinutes.remainder(60).toString().padLeft(2, '0');
  final seconds = d.inSeconds.remainder(60).toString().padLeft(2, '0');
  return "$minutes:$seconds";
}