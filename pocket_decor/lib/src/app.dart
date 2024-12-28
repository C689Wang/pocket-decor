import 'package:pocket_decor/src/settings/settings_controller.dart';
import 'package:flutter/material.dart';
import 'package:pocket_decor/src/login/auth_gate.dart';
import 'package:pocket_decor/src/camera/camera_page.dart';

class MyApp extends StatelessWidget {
  final SettingsController settingsController;

  const MyApp({required this.settingsController, super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Firebase Login',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: AuthGate(),
      routes: {
        '/camera': (context) => CameraPage(),
      },
    );
  }
}
