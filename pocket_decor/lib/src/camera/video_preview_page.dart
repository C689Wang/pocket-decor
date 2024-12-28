import 'dart:io';
import 'dart:convert';
import 'package:flutter/cupertino.dart';
import 'package:video_player/video_player.dart';
import 'package:http/http.dart' as http;
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:firebase_auth/firebase_auth.dart';

class VideoPreviewPage extends StatefulWidget {
  final String filePath;

  const VideoPreviewPage({Key? key, required this.filePath}) : super(key: key);

  @override
  _VideoPreviewPageState createState() => _VideoPreviewPageState();
}

class _VideoPreviewPageState extends State<VideoPreviewPage> {
  late VideoPlayerController _videoPlayerController;
  late TextEditingController _textController;
  bool _isUploading = false;

  @override
  void initState() {
    super.initState();
    _textController = TextEditingController(text: 'Video Title');
    _videoPlayerController = VideoPlayerController.file(File(widget.filePath))
      ..initialize().then((_) {
        setState(() {});
        _videoPlayerController.setLooping(true);
        _videoPlayerController.play();
      });
  }

  @override
  void dispose() {
    _videoPlayerController.dispose();
    _textController.dispose();
    super.dispose();
  }

  String _sanitizePublicId(String input) {
    return input
        .trim()
        .toLowerCase()
        .replaceAll(RegExp(r'[^\w\s-]'), '')
        .replaceAll(RegExp(r'\s+'), '_');
  }

  Future<void> _uploadVideo() async {
    if (_isUploading) return;

    setState(() {
      _isUploading = true;
    });

    try {
      final cloudName = dotenv.env['CLOUD_NAME'];
      final uploadPreset = dotenv.env['PRESET_ID'];
      final User? currentUser = FirebaseAuth.instance.currentUser;

      if (cloudName == null || uploadPreset == null) {
        throw Exception('Missing Cloudinary configuration');
      }

      if (currentUser == null) {
        throw Exception('User not authenticated');
      }

      final userId = currentUser.uid;
      final timestamp = DateTime.now().millisecondsSinceEpoch;
      final sanitizedTitle = _sanitizePublicId(_textController.text);
      final publicId = '${sanitizedTitle}_$timestamp';

      print('Uploading with public ID: $publicId');

      final url =
          Uri.parse('https://api.cloudinary.com/v1_1/$cloudName/video/upload');

      // Create multipart request
      final request = http.MultipartRequest('POST', url)
        ..fields['upload_preset'] = uploadPreset
        ..fields['public_id'] = publicId // Using structured public_id
        ..fields['folder'] = 'users/$userId'
        ..fields['context'] = jsonEncode({
          // Add metadata
          'user_id': userId,
          'user_email': currentUser.email,
          'upload_time': DateTime.now().toIso8601String(),
        })
        ..files.add(
          await http.MultipartFile.fromPath(
            'file',
            widget.filePath,
          ),
        );

      final response = await request.send();
      final responseData = await response.stream.toBytes();
      final responseString = String.fromCharCodes(responseData);
      final jsonData = jsonDecode(responseString);

      if (response.statusCode == 200) {
        print('Video uploaded successfully: ${jsonData['secure_url']}');
        // Show success dialog
        if (mounted) {
          showCupertinoDialog(
            context: context,
            builder: (context) => CupertinoAlertDialog(
              title: const Text('Success'),
              content: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('Video uploaded successfully!'),
                  const SizedBox(height: 8),
                  Text('ID: $publicId', style: const TextStyle(fontSize: 12)),
                ],
              ),
              actions: [
                CupertinoDialogAction(
                  child: const Text('OK'),
                  onPressed: () {
                    Navigator.pop(context); // Close dialog
                    Navigator.pop(context); // Return to camera
                  },
                ),
              ],
            ),
          );
        }
      } else {
        throw Exception('Upload failed: ${jsonData['error']['message']}');
      }
    } catch (e) {
      print('Failed to upload video: $e');
      if (mounted) {
        showCupertinoDialog(
          context: context,
          builder: (context) => CupertinoAlertDialog(
            title: const Text('Error'),
            content: Text('Failed to upload video: $e'),
            actions: [
              CupertinoDialogAction(
                child: const Text('OK'),
                onPressed: () => Navigator.pop(context),
              ),
            ],
          ),
        );
      }
    } finally {
      setState(() {
        _isUploading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return CupertinoPageScaffold(
      navigationBar: CupertinoNavigationBar(
        middle: const Text('Preview'),
        backgroundColor: CupertinoColors.black.withOpacity(0.6),
        trailing: _isUploading
            ? const CupertinoActivityIndicator()
            : CupertinoButton(
                padding: EdgeInsets.zero,
                onPressed: _uploadVideo,
                child: const Icon(
                  CupertinoIcons.cloud_upload,
                  color: CupertinoColors.white,
                ),
              ),
        border: null,
      ),
      backgroundColor: CupertinoColors.black,
      child: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: Stack(
                alignment: Alignment.center,
                children: [
                  _videoPlayerController.value.isInitialized
                      ? AspectRatio(
                          aspectRatio: _videoPlayerController.value.aspectRatio,
                          child: VideoPlayer(_videoPlayerController),
                        )
                      : const CupertinoActivityIndicator(),
                  GestureDetector(
                    onTap: () {
                      setState(() {
                        if (_videoPlayerController.value.isPlaying) {
                          _videoPlayerController.pause();
                        } else {
                          _videoPlayerController.play();
                        }
                      });
                    },
                  ),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(16.0),
              child: CupertinoTextField(
                controller: _textController,
                placeholder: 'Enter video title',
                style: const TextStyle(color: CupertinoColors.white),
                decoration: BoxDecoration(
                  border: Border.all(color: CupertinoColors.white),
                  borderRadius: BorderRadius.circular(8),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
