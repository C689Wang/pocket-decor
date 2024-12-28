import 'package:camera/camera.dart';
import 'package:flutter/cupertino.dart';
import 'package:pocket_decor/src/camera/video_preview_page.dart';
import 'package:flutter/services.dart';
import 'dart:io';
import 'package:flutter/foundation.dart';

class CameraPage extends StatefulWidget {
  const CameraPage({super.key});
  @override
  CameraPageState createState() => CameraPageState();
}

class CameraPageState extends State<CameraPage> {
  CameraController? _cameraController;
  List<CameraDescription>? cameras;
  bool isRecording = false;

  @override
  void initState() {
    super.initState();
    initializeCamera();
  }

  Future<void> initializeCamera() async {
    cameras = await availableCameras();
    if (cameras != null && cameras!.isNotEmpty) {
      _cameraController = CameraController(
        cameras!.first,
        ResolutionPreset.high,
      );
      await _cameraController?.initialize();
      setState(() {});
    }
  }

  @override
  void dispose() {
    _cameraController?.dispose();
    super.dispose();
  }

  Future<void> startRecording() async {
    if (_cameraController != null &&
        !_cameraController!.value.isRecordingVideo) {
      setState(() {
        isRecording = true;
      });
      await _cameraController?.startVideoRecording();
    }
  }

  Future<void> stopRecording() async {
    if (_cameraController != null &&
        _cameraController!.value.isRecordingVideo) {
      final videoFile = await _cameraController?.stopVideoRecording();
      setState(() {
        isRecording = false;
      });
      if (videoFile != null && mounted) {
        Navigator.push(
          context,
          CupertinoPageRoute(
            builder: (context) => VideoPreviewPage(filePath: videoFile.path),
          ),
        );
      }
    }
  }

  void _launchTestPreview() async {
    // This is an example path for iOS simulator
    final testVideoPath = '/tmp/test_video.mp4';

    // Copy test video from assets to temporary directory
    final ByteData data = await rootBundle.load('assets/videos/test_video.MOV');
    final bytes = data.buffer.asUint8List();
    await File(testVideoPath).writeAsBytes(bytes);

    if (mounted) {
      Navigator.push(
        context,
        CupertinoPageRoute(
          builder: (context) => VideoPreviewPage(filePath: testVideoPath),
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return CupertinoPageScaffold(
      navigationBar: CupertinoNavigationBar(
        middle: Text('Record Video'),
        trailing: CupertinoButton(
          padding: EdgeInsets.zero,
          child: Icon(
            CupertinoIcons.clear,
            color: CupertinoColors.systemRed,
          ),
          onPressed: () => Navigator.pop(context),
        ),
      ),
      child: Stack(
        children: [
          // Camera preview or loading indicator
          _cameraController == null || !_cameraController!.value.isInitialized
              ? Center(
                  child: CupertinoActivityIndicator(),
                )
              : Stack(
                  children: [
                    CameraPreview(_cameraController!),
                    Align(
                      alignment: Alignment.bottomCenter,
                      child: Padding(
                        padding: const EdgeInsets.all(20.0),
                        child: CupertinoButton.filled(
                          borderRadius: BorderRadius.circular(30),
                          onPressed:
                              isRecording ? stopRecording : startRecording,
                          child: Icon(
                            isRecording
                                ? CupertinoIcons.stop_fill
                                : CupertinoIcons.videocam_fill,
                            size: 28,
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
          // Test button always visible in debug mode
          if (kDebugMode)
            Positioned(
              bottom: 100,
              right: 20,
              child: CupertinoButton.filled(
                onPressed: _launchTestPreview,
                child: Text('Test Upload'),
              ),
            ),
        ],
      ),
    );
  }
}
