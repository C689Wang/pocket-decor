import 'package:flutter/material.dart';
import 'package:firebase_auth/firebase_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';

class LoginPage extends StatelessWidget {
  final FirebaseAuth _auth = FirebaseAuth.instance;
  final GoogleSignIn _googleSignIn = GoogleSignIn();

  LoginPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: OutlinedButton(
          style: OutlinedButton.styleFrom(
            padding: EdgeInsets.symmetric(vertical: 16.0, horizontal: 32.0),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(30.0),
            ),
            side: BorderSide(color: Colors.blue, width: 2),
            backgroundColor: Colors.white,
          ),
          onPressed: () async {
            try {
              final GoogleSignInAccount? googleUser =
                  await _googleSignIn.signIn();
              final GoogleSignInAuthentication? googleAuth =
                  await googleUser?.authentication;

              final AuthCredential credential = GoogleAuthProvider.credential(
                accessToken: googleAuth?.accessToken,
                idToken: googleAuth?.idToken,
              );
              await _auth.signInWithCredential(credential);
              // if (!context.mounted) return;
              // Navigator.pushNamed(context, '/home');
            } catch (e) {
              // Handle error
              print(e);
            }
          },
          child: Text(
            'Sign in with Google',
            style: TextStyle(
              color: Colors.blue,
              fontSize: 18,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ),
    );
  }
}
