import 'package:flutter/material.dart';

class GestureControlPage extends StatefulWidget{
  const GestureControlPage({super.key});

  @override
  State<StatefulWidget> createState() => _GestureControlPageState();
}

class _GestureControlPageState extends State<GestureControlPage>{

  double sliderVal = 0;

  @override
  Widget build(BuildContext context) {
    return  Column(
      children: [
        Align(
          alignment: Alignment.topRight,
          child: IconButton(
            icon: Icon(Icons.arrow_back),
            color: Colors.white,
            onPressed: (){
              Navigator.of(context).pop();
            },
          ),
        ),
        Expanded(
            child: ListView(
              padding: const EdgeInsets.all(16),
              children: [
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:  ///Top row: label + image
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "Up",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 12,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 12,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "Down",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "Left",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "Right",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "Left Rotation",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "Right Rotation",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "land",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "Take Off",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
                ListTile(
                  contentPadding: EdgeInsets.symmetric(horizontal: 16.0),
                  title:
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(width: 22,), ///Spacer

                      Text(
                        "Flip",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      Image.asset(
                        'assets/Images/Ges.png',
                        height: 80,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider label
                      Text(
                        "Set Distance",
                        style: Theme.of(context).textTheme.headlineLarge,
                      ),

                      const SizedBox(width: 22,), ///Spacer

                      ///Slider
                      ConstrainedBox(
                        constraints: const BoxConstraints(
                            maxWidth: 350
                        ),
                        child: Slider(
                          value: sliderVal,
                          max: 100,
                          divisions: 5,
                          onChanged: (double value){
                            setState(() {
                              sliderVal = value;
                            });
                          },
                        ),
                      )
                    ],
                  ),
                ),
              ],
            )
        )
      ],
    );
  }
}