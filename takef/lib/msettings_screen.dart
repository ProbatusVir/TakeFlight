import 'package:flutter/material.dart';

class MsettingsScreen extends StatelessWidget {
  const MsettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      drawer: Drawer(
        child: ListView(
          padding: EdgeInsets.zero,
          children: [
            ListTile(
              title: const Text('Personalization'),
              onTap: () {
                Navigator.pop(context);
                Navigator.pushNamed(context, '/personalization');
              },
            ),
            ListTile(
              title: const Text('Drone Info'),
                onTap: () {
                  Navigator.pop(context);
                  Navigator.pushNamed(context, '/drone-info');
                }
            ),
            ListTile(
              title: const Text('Gesture Control'),
              onTap: () => Navigator.pushNamed(context, '/gesture-control'),
            ),
            ListTile(
              title: const Text('Flight Logs'),
              onTap: () => Navigator.pushNamed(context, '/flight-logs'),
            ),
          ],
        ),
      ),
      appBar: AppBar(
        leading: Builder(
          builder: (context){
            return IconButton(
                onPressed: () {
                  Scaffold.of(context).openDrawer();
                },
                icon: Icon(Icons.menu)
            );
          },
        ),
      ),
    );
  }
}