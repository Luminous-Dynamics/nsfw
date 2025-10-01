#!/usr/bin/env bash
#
# Download Windows 11 Evaluation ISO
# Free for 90 days of testing
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ISO_DIR="$SCRIPT_DIR/iso"
ISO_FILE="$ISO_DIR/Win11_EnglishInternational_x64.iso"

echo "🪟 Windows 11 ISO Download Script"
echo "=================================="
echo ""

# Create ISO directory
mkdir -p "$ISO_DIR"

# Check if ISO already exists
if [ -f "$ISO_FILE" ]; then
    echo "✅ Windows 11 ISO already exists at:"
    echo "   $ISO_FILE"
    echo ""
    ls -lh "$ISO_FILE"
    echo ""
    echo "Delete this file if you want to re-download."
    exit 0
fi

echo "📥 Downloading Windows 11 Evaluation ISO..."
echo ""
echo "⚠️  IMPORTANT: This is a manual process"
echo ""
echo "Microsoft doesn't allow direct command-line downloads."
echo "You need to download manually from:"
echo ""
echo "🔗 https://www.microsoft.com/en-us/evalcenter/evaluate-windows-11-enterprise"
echo ""
echo "Steps:"
echo "1. Visit the URL above"
echo "2. Click 'Download the ISO - Enterprise'"
echo "3. Fill out the form (or use fake data)"
echo "4. Select 'English (United States)' or your language"
echo "5. Download the 64-bit ISO"
echo "6. Move the downloaded ISO to: $ISO_DIR/"
echo "7. Rename it to: Win11_EnglishInternational_x64.iso"
echo ""
echo "Alternative: Windows 11 Media Creation Tool"
echo "If you have access to Windows, use:"
echo "https://www.microsoft.com/software-download/windows11"
echo ""

# Try wget with user confirmation (in case Microsoft allows it)
echo "Attempt automatic download? (This usually fails)"
read -p "Try anyway? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Attempting download (this will likely fail)..."

    # This URL is for Windows 10, but kept as example
    # Windows 11 doesn't have a direct download URL
    DOWNLOAD_URL="https://software-download.microsoft.com/download/pr/Win11_22H2_EnglishInternational_x64.iso"

    if command -v wget &> /dev/null; then
        wget -O "$ISO_FILE.download" "$DOWNLOAD_URL" && mv "$ISO_FILE.download" "$ISO_FILE" || {
            echo "❌ Automatic download failed (expected)"
            echo "Please follow manual steps above."
            rm -f "$ISO_FILE.download"
        }
    else
        echo "❌ wget not found. Install with: nix-shell -p wget"
    fi
fi

# Check if user downloaded it
if [ -f "$ISO_FILE" ]; then
    echo ""
    echo "✅ Success! ISO found at: $ISO_FILE"
    echo "   Size: $(du -h "$ISO_FILE" | cut -f1)"
    echo ""
    echo "Next step: Run ./create-vm.sh"
else
    echo ""
    echo "⏳ Waiting for manual download..."
    echo ""
    echo "Once downloaded, place the ISO at:"
    echo "   $ISO_FILE"
    echo ""
    echo "Then run: ./create-vm.sh"
fi
