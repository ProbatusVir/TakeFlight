import 'package:flutter/material.dart';

void disconnect (BuildContext context){
  showDialog(
      context: context,
      barrierDismissible: false,
      builder: (BuildContext context){
        return AlertDialog(
          title: const Text('Drone Connection Lost'),
          content: Text('Unable to receive drone data. Please retry or exit back to home page.'),
          actions: [
            TextButton(
                onPressed: (){
                  //TODO::Needs to retry with the selected SSID
                  Navigator.of(context).pop();
                },
                child: const Text("Retry")
            ),
            TextButton(
                onPressed: (){
                  Navigator.of(context).pop();
                  Navigator.of(context).pushNamedAndRemoveUntil(
                    '/home',
                        (route) => false,
                  );
                },
                child: const Text('Exit')
            )
          ],
        );
      }
  );
}