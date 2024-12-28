import os
import ffmpeg
from cloudinary import CloudinaryResource
import cloudinary
import cloudinary.api
from dotenv import load_dotenv
import inquirer
import requests
import subprocess

# Load environment variables
load_dotenv()

# Configure Cloudinary
cloudinary.config(
    cloud_name=os.getenv('CLOUDINARY_CLOUD_NAME'),
    api_key=os.getenv('CLOUDINARY_API_KEY'),
    api_secret=os.getenv('CLOUDINARY_API_SECRET')
)

def create_directory(path):
    """Create directory if it doesn't exist"""
    if not os.path.exists(path):
        os.makedirs(path)

def extract_frames(video_path, frames_dir, frame_interval=1):
    """
    Extract frames from video using ffmpeg
    frame_interval: extract one frame every N seconds
    """
    try:
        # Get video information
        probe = ffmpeg.probe(video_path)
        video_info = next(s for s in probe['streams'] if s['codec_type'] == 'video')
        duration = float(probe['format']['duration'])
        
        # Calculate frame extraction rate
        frame_rate = f"1/{frame_interval}"
        
        # Output pattern for frames
        output_pattern = os.path.join(frames_dir, "frame_%04d.jpg")
        
        # Extract frames using ffmpeg
        stream = ffmpeg.input(video_path)
        stream = ffmpeg.filter(stream, 'fps', fps=frame_rate)
        stream = ffmpeg.output(stream, output_pattern, format='image2', qscale=3)
        ffmpeg.run(stream, capture_stdout=True, capture_stderr=True)
        
        # Calculate approximate number of frames extracted
        num_frames = int(duration / frame_interval)
        return num_frames
        
    except ffmpeg.Error as e:
        print(f"FFmpeg error occurred: {str(e.stderr.decode())}")
        return 0

def download_video(public_id, save_path):
    """
    Download video from Cloudinary using the generated URL
    """
    try:
        # Generate the video URL
        url = cloudinary.utils.cloudinary_url(public_id, resource_type="video")[0]
        
        # Download the video using requests
        response = requests.get(url, stream=True)
        response.raise_for_status()
        
        # Save the video file
        with open(save_path, 'wb') as f:
            for chunk in response.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)
                    
        return True
    except Exception as e:
        print(f"Error downloading video: {str(e)}")
        return False

def run_object_capture(frames_dir, output_dir, video_name):
    """
    Run the pocketDecorObjectCapture executable to generate 3D model
    """
    try:
        # Specify output filename with .usdz extension, using original video name
        base_name = video_name.rsplit('.', 1)[0]  # Remove video extension
        output_file = os.path.join(output_dir, f"{base_name}.usdz")
        
        # Construct the command
        command = [
            './pocketDecorObjectCapture',
            frames_dir,
            output_file,
            '--detail', 'medium',
            '--sample-ordering', 'sequential'
        ]
        
        # Run the command
        result = subprocess.run(command, capture_output=True, text=True)
        
        if result.returncode == 0:
            print("Successfully generated 3D model")
            return True
        else:
            print(f"Error generating 3D model: {result.stderr}")
            return False
            
    except Exception as e:
        print(f"Error running object capture: {str(e)}")
        return False

def fetch_user_videos():
    # Get user input
    user_id = input("Please enter the user ID: ")
    
    # Create base output directory
    output_dir = "downloaded_videos"
    user_dir = os.path.join(output_dir, user_id)
    create_directory(user_dir)
    
    try:
        # Search for all resources with prefix users/{user_id}/
        prefix = f"users/{user_id}/"
        result = cloudinary.api.resources(
            type="upload",
            prefix=prefix,
            resource_type="video",
            max_results=500
        )
        
        if not result.get('resources'):
            print(f"No videos found for user {user_id}")
            return
        
        # Create list of video choices
        choices = [resource['public_id'].split('/')[-1] for resource in result['resources']]
        
        # Create the selection prompt
        questions = [
            inquirer.List('video',
                         message="Which video would you like to process?",
                         choices=choices,
                         carousel=True)
        ]
        
        # Get user selection
        answers = inquirer.prompt(questions)
        selected_video = answers['video']
        
        # Find the selected resource
        selected_resource = next(r for r in result['resources'] 
                               if r['public_id'].split('/')[-1] == selected_video)
        
        # Create video and frames directories
        video_dir = os.path.join(user_dir, selected_video.rsplit('.', 1)[0])
        create_directory(video_dir)
        frames_dir = os.path.join(video_dir, 'frames')
        create_directory(frames_dir)
        
        # Download the video
        video_path = os.path.join(video_dir, selected_video)
        if download_video(selected_resource['public_id'], video_path):
            print(f"Downloaded: {selected_video}")
            
            # Extract frames
            num_frames = extract_frames(video_path, frames_dir, frame_interval=1)
            print(f"Extracted approximately {num_frames} frames from {selected_video}")
            
            # Create models directory
            models_base_dir = "models"
            create_directory(models_base_dir)
            user_models_dir = os.path.join(models_base_dir, user_id)
            create_directory(user_models_dir)
            model_output_dir = os.path.join(user_models_dir, selected_video.rsplit('.', 1)[0])
            create_directory(model_output_dir)
            
            # Generate 3D model
            # if run_object_capture(frames_dir, model_output_dir, selected_video):
            #     print(f"\nModel has been generated and saved in: {model_output_dir}")
            # else:
            #     print("\nFailed to generate 3D model")
            
            # print(f"\nVideo has been downloaded and processed in: {video_dir}")
        else:
            print("Failed to download video")
            
    except Exception as e:
        print(f"An error occurred: {str(e)}")

if __name__ == "__main__":
    fetch_user_videos() 