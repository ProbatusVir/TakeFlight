import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'dart:math';

class FlightLogsPage extends StatefulWidget{
  const FlightLogsPage({super.key});

  @override
  State<FlightLogsPage> createState() => _FlightLogsPageState();
}

class _FlightLogsPageState extends State<FlightLogsPage>{
  //TODO::Replace fakeLogs with real once packet handling is better
  final List<String> fakeLogs = List.generate(30, (_){
    final date = randomDateTime(
      start: DateTime(2025, 1, 1),
      end: DateTime.now()
    );
    return DateFormat('M/d/yy hh:mm a').format(date);
  });

  String? selectedLog;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child : Column(
            children: [
              Text(
                "Flight Logs",
                style: Theme.of(context).textTheme.headlineLarge,
              ),
              Expanded(
                  child: ListView.builder(
                    itemCount: fakeLogs.length,
                      itemBuilder: (context, index){
                      final log = fakeLogs[index];
                      return ListTile(
                        title: Text(log, style: Theme.of(context).textTheme.bodyLarge,),
                        selected: log == selectedLog,
                        onTap: (){
                          setState(() {
                            selectedLog = log;
                          });
                        },
                      );
                    }
                  )
              ),
            ],
          ),
        ),
        ///Log details
        Expanded(
            child: selectedLog == null
                ? Center(
              child: Text("Select a flight log",style: Theme.of(context).textTheme.bodyLarge),
            )
                :
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(selectedLog!,
                      style: Theme.of(context).textTheme.headlineLarge
                  ),
                  const SizedBox(height: 16,),
                  //TODO::Actual drone log entries will be placed here
                  const Text("• Altitude: 120m"),
                  const Text("• Duration: 00:03:42"),
                  const Text("• Battery: 87%"),
                ],
              ),
            )
        ),
      ],
    );
  }
}

//helper tool
DateTime randomDateTime({
  required DateTime start,
  required DateTime end,
}) {
  final random = Random();
  final range = end.millisecondsSinceEpoch - start.millisecondsSinceEpoch;
  final randomMillis = start.millisecondsSinceEpoch + (random.nextDouble() * range).toInt();
  return DateTime.fromMillisecondsSinceEpoch(randomMillis);
}