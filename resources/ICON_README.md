# App Icon Guide

This directory contains the app icon resources for ClipVault.

## Current Status

**Icon Status**: Placeholder / Not Yet Created

A professional app icon is needed for distribution. This document outlines the requirements.

## Icon Requirements

### Source Requirements

- **Format**: PNG or vector (AI/Sketch/Figma)
- **Size**: 1024x1024 pixels minimum
- **Color**: Full color (not monochrome)
- **Background**: Can be transparent or solid
- **Style**: macOS Big Sur+ style (rounded square with padding)

### Design Guidelines

**Recommended**:
- Simple, recognizable design
- Works well at small sizes (16x16)
- Consistent with ClipVault branding
- macOS visual language (depth, shadows)

**Theme Ideas**:
- Clipboard with lock (privacy focus)
- Stacked papers/items (history)
- Abstract "V" shape (ClipVault)
- Minimalist clipboard icon

**Colors**:
- Primary: Blue/Purple (trust, security)
- Accent: Gold/Yellow (premium)
- Avoid: Red (error), Grey (too plain)

### Icon Set Structure

Required sizes for `.icns` file:

```
AppIcon.iconset/
├── icon_16x16.png       # 16x16 pixels
├── icon_16x16@2x.png    # 32x32 pixels
├── icon_32x32.png       # 32x32 pixels
├── icon_32x32@2x.png    # 64x64 pixels
├── icon_128x128.png     # 128x128 pixels
├── icon_128x128@2x.png  # 256x256 pixels
├── icon_256x256.png     # 256x256 pixels
├── icon_256x256@2x.png  # 512x512 pixels
├── icon_512x512.png     # 512x512 pixels
└── icon_512x512@2x.png  # 1024x1024 pixels
```

## Creating the Icon

### Step 1: Design

Use a design tool:
- **Figma**: Free, browser-based
- **Sketch**: macOS native (paid)
- **Photoshop/Illustrator**: Adobe Creative Cloud

### Step 2: Export Sizes

From your 1024x1024 source, export all required sizes:

```bash
# Using ImageMagick (install: brew install imagemagick)
convert icon-1024.png -resize 16x16     icon_16x16.png
convert icon-1024.png -resize 32x32     icon_16x16@2x.png
convert icon-1024.png -resize 32x32     icon_32x32.png
convert icon-1024.png -resize 64x64     icon_32x32@2x.png
convert icon-1024.png -resize 128x128   icon_128x128.png
convert icon-1024.png -resize 256x256   icon_128x128@2x.png
convert icon-1024.png -resize 256x256   icon_256x256.png
convert icon-1024.png -resize 512x512   icon_256x256@2x.png
convert icon-1024.png -resize 512x512   icon_512x512.png
convert icon-1024.png -resize 1024x1024 icon_512x512@2x.png
```

Or use `sips` (built-in on macOS):

```bash
# Example for one size
sips -z 16 16 icon-1024.png --out icon_16x16.png
```

### Step 3: Create Iconset Directory

```bash
mkdir AppIcon.iconset
# Move all exported icons into AppIcon.iconset/
```

### Step 4: Generate .icns File

```bash
# Convert iconset to .icns
iconutil -c icns AppIcon.iconset

# Output: AppIcon.icns
```

### Step 5: Verify Icon

```bash
# Preview the icon
qlmanage -p AppIcon.icns

# Or drag AppIcon.icns onto Preview app
```

## Using the Icon

### In App Bundle

Copy `AppIcon.icns` to:
```
ClipVault.app/Contents/Resources/AppIcon.icns
```

### In Info.plist

Ensure `Info.plist` references it:
```xml
<key>CFBundleIconFile</key>
<string>AppIcon</string>
```

(Note: No `.icns` extension in plist)

## Temporary/Placeholder Icon

For development, you can use macOS system icons:

```bash
# Copy generic app icon
cp /System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns \
   resources/AppIcon.icns
```

**Note**: Do not ship with generic icon! Replace before distribution.

## Icon Generator Tools

### Online Tools
- [Icon Generator](https://apps.apple.com/us/app/icon-generator/id1490879979) (Mac App)
- [App Icon Generator](https://appicon.co/) (Web)
- [MakeAppIcon](https://makeappicon.com/) (Web)

### Design Services
- [Fiverr](https://fiverr.com) - $5-50 for icon design
- [99designs](https://99designs.com) - Icon design contests
- [Dribbble](https://dribbble.com) - Hire designers

## Icon Checklist

Before shipping:

- [ ] Icon designed at 1024x1024
- [ ] All sizes exported (16 to 1024)
- [ ] Iconset directory created
- [ ] .icns file generated
- [ ] Icon tested at all sizes
- [ ] Icon looks good in menu bar
- [ ] Icon looks good in Finder
- [ ] Icon looks good in Dock (if applicable)
- [ ] Icon matches brand identity

## References

- [Apple HIG - App Icon](https://developer.apple.com/design/human-interface-guidelines/app-icons)
- [macOS Big Sur Icon Template](https://applypixels.com/template/macos-big-sur/)
- [Icon Design Best Practices](https://bjango.com/articles/icondesign/)

---

**Current Status**: Awaiting icon design

**Priority**: Medium (required before v1.0 release, placeholder OK for beta)

**Next Steps**:
1. Sketch icon concepts
2. Create 1024x1024 source
3. Export all sizes
4. Generate .icns file
5. Test in app bundle
