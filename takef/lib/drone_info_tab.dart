import 'package:flutter/material.dart';

import 'connect.dart';

class DroneInfoPage extends StatefulWidget{
  const DroneInfoPage({super.key, required this.info});
  final Map<String, dynamic> info;

  @override
  State<DroneInfoPage> createState() => _DroneInfoPageState();
}
///TODO::Should more than likely fix up the small sizing of words rn its better
class _DroneInfoPageState extends State<DroneInfoPage>{

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Top row
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Drone1:',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.bold,
                    fontSize: 30,
                  ),
                ),
                IconButton(
                  icon: Icon(Icons.arrow_back),
                  color: Colors.white,
                  onPressed: () => Navigator.pop(context),
                ),
              ],
            ),

            const SizedBox(height: 16),

            // Stats
            Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: const [
                _StatRow(icon: Icons.battery_4_bar, text: 'Battery Status: 78%', color: Colors.green,),
                _StatRow(icon: Icons.thermostat, text: 'Temperature: 35°C'),
                _StatRow(icon: Icons.access_time, text: 'Flight Time: 12m 34s'),
              ],
            ),

            const SizedBox(height: 20),

            // Description
            Text(
              'Description:',
              style: TextStyle(color: Colors.white, fontSize: 24),
            ),
            const SizedBox(height: 8),

            Text(
              '• Model info: Learn about drones...\n'
                  '• 5-megapixel camera records JPEG and 720p MP4...\n'
                  '• Operates up to 13 minutes per charge...\n'
                  '• DJI flight tech ensures stable flights...\n'
                  '• VR headset compatibility, FOV 82.6°',
              style: TextStyle(color: Colors.white70, fontSize: 16),
            ),
          ],
        ),
      ),
    );
  }
}

class _StatRow extends StatelessWidget{
  const _StatRow({super.key, required this.icon, required this.text, this.color = Colors.white});
  final IconData icon;
  final String text;
  final Color color;
  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6.0),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Icon(
            icon,
            size: 22,
            color: color,
          ),
          const SizedBox(width: 12),
          Text(
            text,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 20,
            ),
          ),
        ],
      ),
    );
  }
}