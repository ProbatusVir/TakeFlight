import 'package:flutter/material.dart';
import 'flight_control_mode.dart';
import 'package:takef/central_screen.dart';

class PersonalizationPage extends StatefulWidget {
  const PersonalizationPage({super.key});

  @override
  State<PersonalizationPage> createState() => _PersonalizationPageState();
}

class _PersonalizationPageState extends State<PersonalizationPage>{
  bool isLightTheme = false;
  ControlMode selectedMode = ControlMode.joystick;

  Future<void> noOverride() async{
    final loaded =  await loadControlMode();

    if(!mounted) return;

    setState(() {
      selectedMode = loaded;
    });
  }

  @override
  void initState() {
    super.initState();
    noOverride();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
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
        Text("Layout Settings", style: Theme.of(context).textTheme.headlineLarge),
        Text("Change Layout of onscreen drone controls", style: Theme.of(context).textTheme.bodyLarge),
        ///Mini Preview
        Expanded(
            child: SizedBox(
              width: 700,
              height: 350,
              child: ClipRRect(
                borderRadius: BorderRadius.circular(12),
                child: MediaQuery(
                    data: MediaQuery.of(context).copyWith(
                      size: const Size(390, 844)
                    ),
                  ///TODO::Should replace with a reactive test screen
                    child: Scaffold(
                      backgroundColor: Colors.grey[600],
                      body: Center(
                        child: Text("Drone Controls"),
                      ),
                    ),
                ),
              ),
            )
        ),
        Text("Toggle Light Theme", style: Theme.of(context).textTheme.headlineLarge),
        Switch(
            value: isLightTheme,
            onChanged: (value){
              setState(() {
                isLightTheme = value;
              });
            },
          activeThumbColor: Colors.white,
          activeTrackColor: Colors.grey.shade600,
          inactiveThumbColor: Colors.white,
          inactiveTrackColor: Colors.redAccent,
        ),
        Text("Control Mode", style: Theme.of(context).textTheme.headlineLarge),
        Switch(
          value: selectedMode == ControlMode.keyboard,
          onChanged: (enabled) async{
            setState(() {
              selectedMode = enabled ? ControlMode.keyboard
                  : ControlMode.joystick;
            });

            debugPrint("Switch enabled = $enabled");
            debugPrint("selectedMode BEFORE save = $selectedMode");

            await saveControlMode(selectedMode);
          },
          activeThumbColor: Colors.white,
          activeTrackColor: Colors.grey.shade600,
          inactiveThumbColor: Colors.white,
          inactiveTrackColor: Colors.redAccent,
        ),
        Text("File Settings", style: Theme.of(context).textTheme.headlineLarge),
        //TODO::Not sure what it would this setting would pertain to as of rn
        Text("bla bla bla", style: Theme.of(context).textTheme.bodyLarge),
      ],
    );
  }
}