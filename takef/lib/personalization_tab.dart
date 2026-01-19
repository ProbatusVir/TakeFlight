import 'package:flutter/material.dart';

class PersonalizationPage extends StatefulWidget {
  const PersonalizationPage({super.key});

  @override
  State<PersonalizationPage> createState() => _PersonalizationPageState();
}

class _PersonalizationPageState extends State<PersonalizationPage>{

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Text('Personalization content'),
    );
  }
}