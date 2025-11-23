from PIL import Image
import os

icon_dir = "src-tauri/icons"
files_to_fix = ["icon.png", "32x32.png", "128x128.png", "128x128@2x.png"]

for filename in files_to_fix:
    path = os.path.join(icon_dir, filename)
    if os.path.exists(path):
        try:
            print(f"Processing {filename}...")
            img = Image.open(path)
            print(f"  Original mode: {img.mode}, size: {img.size}")
            
            # Force convert to RGBA
            img = img.convert("RGBA")
            
            # Save it back
            img.save(path, "PNG")
            print(f"  Saved as RGBA PNG.")
        except Exception as e:
            print(f"  Error processing {filename}: {e}")
    else:
        print(f"  {filename} not found.")
