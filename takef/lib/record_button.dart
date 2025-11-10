import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

class RecordButton extends StatefulWidget{
  const RecordButton({super.key});
  @override
  _RecordButtonState createState() => _RecordButtonState();
}

class _RecordButtonState extends State<RecordButton>{
  bool isRecording = true;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: (){
        setState((){
          if(isRecording == true){
            isRecording = false;
          }else{
            isRecording = true;
          }
        });
      },
      child: isRecording? SvgPicture.asset(
        'assets/Images/record_icon.svg',
        width: 50,
        height: 50,
        semanticsLabel: 'Record',
      )
          :SvgPicture.asset(
        'assets/Images/Stop_Circle.svg',
        width: 50,
        height: 50,
        semanticsLabel: 'Stop Recording',
      ),
    );
  }
}