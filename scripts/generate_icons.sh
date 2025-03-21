#!/bin/bash

# Help function
show_help() {
    echo "Usage: ./generate_icons.sh [OPTIONS] <source-image>"
    echo
    echo "Generate all necessary icons for a Tauri application from a source image."
    echo "The source image should be at least 1024x1024 pixels for best results."
    echo
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo
    echo "Example:"
    echo "  ./generate_icons.sh ../logo.png     Generate icons from logo.png"
    echo "  ./generate_icons.sh --help          Show this help message"
    echo
    echo "The script will generate:"
    echo "  - macOS icons (.icns and various sizes)"
    echo "  - Windows icons (.ico and various sizes for Windows Store, including 54x54 StoreLogo.png)"
    echo "  - PNG icons for Linux and general use"
    echo
    echo "All icons will be placed in the src-tauri/icons directory."
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            SOURCE_IMAGE="$1"
            break
            ;;
    esac
done

# Check if source image is provided
if [ -z "$SOURCE_IMAGE" ]; then
    echo "Error: No source image provided"
    echo "Run './generate_icons.sh --help' for usage information"
    exit 1
fi

# Check if source image exists
if [ ! -f "$SOURCE_IMAGE" ]; then
    echo "Error: Source image '$SOURCE_IMAGE' not found"
    exit 1
fi

# Check if ImageMagick is installed
if ! command -v magick &> /dev/null; then
    echo "Error: ImageMagick is not installed. Please install it using:"
    echo "brew install imagemagick"
    exit 1
fi

# Get image dimensions
dimensions=$(magick identify -format "%wx%h" "$SOURCE_IMAGE")
width=$(echo $dimensions | cut -d'x' -f1)
height=$(echo $dimensions | cut -d'x' -f2)

# Check image dimensions
if [ "$width" -lt 1024 ] || [ "$height" -lt 1024 ]; then
    echo "Warning: Image dimensions are ${width}x${height}px"
    echo "For best results, use an image at least 1024x1024px"
    read -p "Do you want to continue? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Create temporary iconset directory
rm -rf icon.iconset
mkdir -p icon.iconset

TAURI_ICONS_DIR="../src-tauri/icons"

# Ensure Tauri icons directory exists
mkdir -p "$TAURI_ICONS_DIR"

echo "🎨 Generating icons from: $SOURCE_IMAGE"

# Generate different sizes for macOS
echo "📱 Generating macOS icons..."
magick "$SOURCE_IMAGE" -resize 16x16 icon.iconset/icon_16x16.png
magick "$SOURCE_IMAGE" -resize 32x32 icon.iconset/icon_16x16@2x.png
magick "$SOURCE_IMAGE" -resize 32x32 icon.iconset/icon_32x32.png
magick "$SOURCE_IMAGE" -resize 64x64 icon.iconset/icon_32x32@2x.png
magick "$SOURCE_IMAGE" -resize 128x128 icon.iconset/icon_128x128.png
magick "$SOURCE_IMAGE" -resize 256x256 icon.iconset/icon_128x128@2x.png
magick "$SOURCE_IMAGE" -resize 256x256 icon.iconset/icon_256x256.png
magick "$SOURCE_IMAGE" -resize 512x512 icon.iconset/icon_256x256@2x.png
magick "$SOURCE_IMAGE" -resize 512x512 icon.iconset/icon_512x512.png
magick "$SOURCE_IMAGE" -resize 1024x1024 icon.iconset/icon_512x512@2x.png

# Generate .icns file
echo "🍎 Generating macOS .icns file..."
iconutil -c icns icon.iconset

# Generate Windows/Linux PNG icons
echo "🪟 Generating Windows/Linux PNG icons..."
magick "$SOURCE_IMAGE" -resize 32x32 "$TAURI_ICONS_DIR/32x32.png"
magick "$SOURCE_IMAGE" -resize 128x128 "$TAURI_ICONS_DIR/128x128.png"
magick "$SOURCE_IMAGE" -resize 256x256 "$TAURI_ICONS_DIR/128x128@2x.png"

# Generate Windows specific sizes
echo "🏪 Generating Windows Store icons..."
magick "$SOURCE_IMAGE" -resize 30x30 "$TAURI_ICONS_DIR/Square30x30Logo.png"
magick "$SOURCE_IMAGE" -resize 44x44 "$TAURI_ICONS_DIR/Square44x44Logo.png"
magick "$SOURCE_IMAGE" -resize 54x54 "$TAURI_ICONS_DIR/StoreLogo.png"
magick "$SOURCE_IMAGE" -resize 71x71 "$TAURI_ICONS_DIR/Square71x71Logo.png"
magick "$SOURCE_IMAGE" -resize 89x89 "$TAURI_ICONS_DIR/Square89x89Logo.png"
magick "$SOURCE_IMAGE" -resize 107x107 "$TAURI_ICONS_DIR/Square107x107Logo.png"
magick "$SOURCE_IMAGE" -resize 142x142 "$TAURI_ICONS_DIR/Square142x142Logo.png"
magick "$SOURCE_IMAGE" -resize 150x150 "$TAURI_ICONS_DIR/Square150x150Logo.png"
magick "$SOURCE_IMAGE" -resize 284x284 "$TAURI_ICONS_DIR/Square284x284Logo.png"
magick "$SOURCE_IMAGE" -resize 310x310 "$TAURI_ICONS_DIR/Square310x310Logo.png"

# Generate icon.png (used as the primary icon)
echo "🖼️  Generating primary icon.png..."
magick "$SOURCE_IMAGE" -resize 512x512 "$TAURI_ICONS_DIR/icon.png"

# Move .icns file to Tauri icons directory
mv icon.icns "$TAURI_ICONS_DIR/icon.icns"

# Generate .ico file for Windows
echo "🪟 Generating Windows .ico file..."
magick "$SOURCE_IMAGE" -define icon:auto-resize=256,128,96,64,48,32,16 "$TAURI_ICONS_DIR/icon.ico"

# Clean up
rm -rf icon.iconset

echo "✅ Icon generation complete! Icons have been placed in $TAURI_ICONS_DIR"
echo "📁 Generated $(ls -1 "$TAURI_ICONS_DIR" | wc -l | tr -d ' ') icons in total" 