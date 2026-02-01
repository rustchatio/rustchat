import requests
import sys
import os

# Configuration
BASE_URL = os.environ.get("RUSTCHAT_URL", "http://localhost:8080")
USERNAME = os.environ.get("RUSTCHAT_USER", "sysadmin")
PASSWORD = os.environ.get("RUSTCHAT_PASS", "sysadmin")
TEAM_NAME = os.environ.get("RUSTCHAT_TEAM", "ad-hoc")
CHANNEL_NAME = os.environ.get("RUSTCHAT_CHANNEL", "town-square")

def get_session():
    s = requests.Session()
    print(f"Logging in as {USERNAME}...")
    r = s.post(f"{BASE_URL}/api/v4/users/login", json={"login_id": USERNAME, "password": PASSWORD})
    if r.status_code != 200:
        print(f"Login failed: {r.status_code} {r.text}")
        sys.exit(1)
    # Token is usually in header 'Token' or cookie. Mattermost uses Token header.
    token = r.headers.get("Token")
    if token:
        s.headers.update({"Authorization": f"Bearer {token}"})
    print("Login successful.")
    return s

def get_channel_id(s, team_name, channel_name):
    print(f"Finding channel {team_name}/{channel_name}...")
    r = s.get(f"{BASE_URL}/api/v4/teams/name/{team_name}/channels/name/{channel_name}")
    if r.status_code != 200:
        print(f"Get channel failed: {r.status_code} {r.text}")
        # Try finding just by channel name if team fails? Or just list public channels.
        sys.exit(1)
    return r.json()['id']

def create_dummy_image():
    # Create a small 1x1 png or similar
    # 1x1 transparent GIF
    return b'GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff!\xf9\x04\x01\x00\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x01D\x00;'

def test_mobile_upload(s, channel_id):
    print("Testing Mobile Upload (Relaxed Multipart)...")
    img_data = create_dummy_image()
    
    # Use a random field name here to simulate mobile client behavior or just generic 'files'
    # But specifically checking if 'image' or other names work as per fix.
    # The fix allows ANY field name that is not empty/reserved.
    files = {
        'some_random_field_name': ('mobile_test.gif', img_data, 'image/gif')
    }
    
    r = s.post(f"{BASE_URL}/api/v4/files", files=files, data={'channel_id': channel_id})
    if r.status_code != 201 and r.status_code != 200:
        print(f"Upload failed: {r.status_code} {r.text}")
        sys.exit(1)
    
    resp = r.json()
    print("Upload response:", resp)
    
    if 'file_infos' not in resp or len(resp['file_infos']) == 0:
        print("No file_infos returned!")
        sys.exit(1)
        
    file_id = resp['file_infos'][0]['id']
    print(f"File ID: {file_id}")
    return file_id

def test_preview_streaming(s, file_id):
    print("Testing Preview Streaming (No Redirects)...")
    # Verify we get 200 OK and content, not 302
    r = s.get(f"{BASE_URL}/api/v4/files/{file_id}/preview", allow_redirects=False)
    
    print(f"Preview Status: {r.status_code}")
    print(f"Content-Type: {r.headers.get('Content-Type')}")
    
    if r.status_code == 302 or r.status_code == 307:
        print("FAIL: Received Redirect for preview!")
        sys.exit(1)
        
    if r.status_code != 200:
        print(f"FAIL: Expected 200 OK, got {r.status_code}")
        # It's possible preview generation failed or was skipped for small image? 
        # But we implemented generation for all images. 
        # Wait, for small images (GIF 1x1), thumbnail might be skipping?
        # Our code: if w > 400 ... else ... -> we generate webp for everything now.
        sys.exit(1)
        
    print("PASS: Preview is streaming.")

def main():
    s = get_session()
    # Need to find a valid channel.
    # Allow user to pass channel ID directly if needed, or find default.
    # For now try to find 'town-square' in 'ad-hoc' or similar.
    # We'll just list teams and pick first one.
    try:
        r = s.get(f"{BASE_URL}/api/v4/users/me/teams")
        teams = r.json()
        if not teams:
            print("No teams found for user.")
            sys.exit(1)
        team = teams[0]
        r = s.get(f"{BASE_URL}/api/v4/teams/{team['id']}/channels")
        channels = r.json()
        channel = next((c for c in channels if c['name'] == 'town-square'), channels[0])
        channel_id = channel['id']
    except Exception as e:
        print(f"Error resolving channel: {e}")
        sys.exit(1)
        
    file_id = test_mobile_upload(s, channel_id)
    test_preview_streaming(s, file_id)
    print("ALL TESTS PASSED")

if __name__ == "__main__":
    main()
