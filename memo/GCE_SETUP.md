# Google Compute Engine Deployment Setup

## Prerequisites Completed ✅
- ✅ gcloud CLI installed
- ✅ SSH key generated: `~/.ssh/gce_mecp`

## Step-by-Step Setup

### Step 1: Authenticate with Google Cloud

Run this command (it will open a browser):
```bash
source ~/google-cloud-sdk/path.bash.inc
gcloud auth login
```

### Step 2: Set Your GCP Project

After authentication, set your project:
```bash
# List your projects
gcloud projects list

# Set your project (replace PROJECT_ID with your actual project ID)
gcloud config set project YOUR_PROJECT_ID
```

### Step 3: Create the GCE Instance

Run these commands (replace variables as needed):
```bash
# Set variables
export PROJECT_ID="your-gcp-project-id"
export INSTANCE_NAME="mecp-server"
export ZONE="us-central1-a"  # Choose your preferred zone
export MACHINE_TYPE="e2-standard-2"  # 2 vCPU, 8GB RAM
export DISK_SIZE="20GB"
export USERNAME="gnufoo"  # Your GCE username (usually your Google account username)

# Get your SSH public key
SSH_KEY=$(cat ~/.ssh/gce_mecp.pub)

# Create the instance with startup script
gcloud compute instances create $INSTANCE_NAME \
  --project=$PROJECT_ID \
  --zone=$ZONE \
  --machine-type=$MACHINE_TYPE \
  --boot-disk-size=$DISK_SIZE \
  --boot-disk-type=pd-standard \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --tags=http-server,https-server \
  --metadata=startup-script='#!/bin/bash
    apt-get update
    apt-get install -y git curl build-essential
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    rustup default stable' \
  --metadata=ssh-keys="$USERNAME:$SSH_KEY"

# Allow HTTP/HTTPS traffic
gcloud compute firewall-rules create allow-http-https \
  --allow tcp:80,tcp:443,tcp:8080,tcp:3000 \
  --source-ranges 0.0.0.0/0 \
  --description "Allow HTTP/HTTPS traffic for MeCP" \
  --project=$PROJECT_ID

# Get the external IP
gcloud compute instances describe $INSTANCE_NAME \
  --zone=$ZONE \
  --project=$PROJECT_ID \
  --format='get(networkInterfaces[0].accessConfigs[0].natIP)'
```

### Step 4: Configure SSH for Cursor Remote Editing

Add this to your `~/.ssh/config`:
```
Host mecp-gce
    HostName YOUR_EXTERNAL_IP
    User gnufoo
    IdentityFile ~/.ssh/gce_mecp
    ForwardAgent yes
    ServerAliveInterval 60
    ServerAliveCountMax 3
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
```

Replace `YOUR_EXTERNAL_IP` with the IP from Step 3.

### Step 5: Connect and Deploy

1. **Connect via Cursor:**
   - Press `F1` → "Remote-SSH: Connect to Host" → Select `mecp-gce`
   - Open folder: `/home/gnufoo/MeCP`

2. **In Cursor's terminal (connected to remote), run:**
```bash
# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install dependencies
sudo apt-get install -y \
    git \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    mysql-client \
    docker.io \
    docker-compose

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add user to docker group
sudo usermod -aG docker $USER
# Note: Log out and back in for this to take effect

# Clone repository
cd ~
git clone https://github.com/gnufoo/MeCP.git
cd MeCP

# Build project
cargo build --release

# Install and start services
./target/release/mecp-cli install
./target/release/mecp-cli start
```

### Step 6: Create Systemd Service

Create `/etc/systemd/system/mecp.service`:
```ini
[Unit]
Description=MeCP Server
After=network.target mysql.service

[Service]
Type=simple
User=gnufoo
WorkingDirectory=/home/gnufoo/MeCP
Environment="PORT=8080"
Environment="RUST_LOG=info"
ExecStart=/home/gnufoo/MeCP/target/release/mecp
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Then:
```bash
sudo systemctl daemon-reload
sudo systemctl enable mecp
sudo systemctl start mecp
sudo systemctl status mecp
```

## Quick Reference

```bash
# Check service status
sudo systemctl status mecp

# View logs
sudo journalctl -u mecp -f

# Restart service
sudo systemctl restart mecp

# Check database services
./target/release/mecp-cli status
```

## Your SSH Public Key

Your SSH public key (already added to instance metadata):
```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKbxlMwmoJDWgfo5rRX6xz58AjTql0fDd+zmQ5PfFBuV gce-mecp
```
