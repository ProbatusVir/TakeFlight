import 'package:flutter/material.dart' hide ConnectionState;
import 'main.dart';

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
                onPressed: () async{
                  Navigator.of(context).pop();

                  final status = await info.retryConnection();

                  if(!context.mounted) return;

                  switch(status){

                    case ConnectionState.connecting:
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text("Reconnecting...")),
                      );
                      break;
                    case ConnectionState.connected:
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text(
                            "Reconnected to drone-${info.currSSID}"
                        )),
                      );
                      break;
                    case ConnectionState.failed:
                      disconnect(context);
                    case ConnectionState.disconnected:
                      // TODO: Handle this case.
                      throw UnimplementedError();
                    case ConnectionState.unavailable:
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text(
                            "Drone-${info.currSSID} is no longer available"
                        )),
                      );
                      break;
                  }
                  print('Retrying drone connection....');
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