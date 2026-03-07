# Self-Hosted Runners Setup Guide

This guide explains how to set up self-hosted runners for the VantisWeb repository.

## Why Self-Hosted Runners?

GitHub Actions on private repositories require a paid plan. Self-hosted runners provide a free alternative that gives you full control over your CI/CD environment.

## Prerequisites

- A server (physical or VM) with:
  - Linux (Ubuntu 22.04+ recommended), Windows, or macOS
  - At least 4GB RAM (8GB+ recommended for builds)
  - 50GB+ disk space
  - Network access to GitHub

## Quick Setup

### 1. Add Runner to Repository

1. Go to your repository on GitHub
2. Navigate to **Settings** > **Actions** > **Runners**
3. Click **New self-hosted runner**
4. Select your operating system
5. Follow the download and configuration instructions

### 2. Linux Setup Example

```bash
# Create a directory for the runner
mkdir actions-runner && cd actions-runner

# Download the latest runner package
curl -o actions-runner-linux-x64-2.321.0.tar.gz \
  https://github.com/actions/runner/releases/download/v2.321.0/actions-runner-linux-x64-2.321.0.tar.gz

# Extract the installer
tar xzf ./actions-runner-linux-x64-2.321.0.tar.gz

# Configure the runner (use token from GitHub UI)
./config.sh --url https://github.com/vantisCorp/VantisWeb --token YOUR_TOKEN

# Run the runner
./run.sh
```

### 3. Install Required Dependencies (Ubuntu)

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WebKit2GTK dependencies
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libcairo2-dev \
  libpango1.0-dev \
  libgdk-pixbuf2.0-dev \
  libatk1.0-dev \
  libsoup-3.0-dev \
  javascriptcoregtk-4.1-dev \
  build-essential \
  pkg-config
```

### 4. Run as Service (Linux)

```bash
sudo ./svc.sh install
sudo ./svc.sh start
```

## Runner Labels

After setup, you can use these labels in your workflows:

```yaml
jobs:
  build:
    runs-on: self-hosted
```

Or create custom labels for different runner types:
- `self-hosted-linux` for Linux runners
- `self-hosted-windows` for Windows runners
- `self-hosted-macos` for macOS runners

## Security Considerations

1. **Runner isolation**: Use dedicated machines/VMs for runners
2. **Network security**: Restrict runner access to necessary services only
3. **Token management**: Regularly rotate runner tokens
4. **Updates**: Keep runner software updated

## Alternative: GitHub Actions Billing

If you prefer using GitHub-hosted runners:

1. Go to **Settings** > **Billing and plans**
2. Enable GitHub Actions for your plan
3. Monthly pricing: 
   - Free: 2,000 minutes/month for private repos
   - Team: 3,000 minutes/month included
   - Enterprise: 50,000 minutes/month

## Troubleshooting

### Runner not picking up jobs
- Check runner is online: `./run.sh` or check service status
- Verify runner labels match workflow requirements
- Check repository settings allow self-hosted runners

### Build failures
- Ensure all dependencies are installed on runner
- Check disk space availability
- Review runner logs for detailed error messages

## Resources

- [GitHub Self-Hosted Runners Documentation](https://docs.github.com/en/actions/hosting-your-own-runners)
- [Runner Repository](https://github.com/actions/runner)
- [VantisWeb CI/CD Pipeline](../.github/workflows/ci.yml)