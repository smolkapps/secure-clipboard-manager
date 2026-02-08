#!/usr/bin/env python3
"""Generate ClipVault app icon - clipboard with shield/lock motif."""

from PIL import Image, ImageDraw, ImageFont
import math
import os

SIZE = 1024
PADDING = 100  # macOS icon padding inside rounded square

def rounded_rect(draw, xy, radius, fill):
    """Draw a rounded rectangle."""
    x0, y0, x1, y1 = xy
    # Corners
    draw.pieslice([x0, y0, x0 + 2*radius, y0 + 2*radius], 180, 270, fill=fill)
    draw.pieslice([x1 - 2*radius, y0, x1, y0 + 2*radius], 270, 360, fill=fill)
    draw.pieslice([x0, y1 - 2*radius, x0 + 2*radius, y1], 90, 180, fill=fill)
    draw.pieslice([x1 - 2*radius, y1 - 2*radius, x1, y1], 0, 90, fill=fill)
    # Edges
    draw.rectangle([x0 + radius, y0, x1 - radius, y1], fill=fill)
    draw.rectangle([x0, y0 + radius, x0 + radius, y1 - radius], fill=fill)
    draw.rectangle([x1 - radius, y0 + radius, x1, y1 - radius], fill=fill)


def create_gradient(img, color1, color2):
    """Apply a vertical gradient to the image."""
    draw = ImageDraw.Draw(img)
    for y in range(img.height):
        t = y / img.height
        r = int(color1[0] + (color2[0] - color1[0]) * t)
        g = int(color1[1] + (color2[1] - color1[1]) * t)
        b = int(color1[2] + (color2[2] - color1[2]) * t)
        draw.line([(0, y), (img.width, y)], fill=(r, g, b))


def generate_icon():
    # Create base image with transparency
    img = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))

    # --- macOS rounded square background ---
    bg = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    # Gradient: deep blue to purple
    grad = Image.new('RGB', (SIZE, SIZE))
    create_gradient(grad, (41, 98, 255), (103, 58, 183))  # Blue to purple
    grad_rgba = grad.convert('RGBA')

    # Create rounded square mask
    mask = Image.new('L', (SIZE, SIZE), 0)
    mask_draw = ImageDraw.Draw(mask)
    corner_radius = int(SIZE * 0.22)  # macOS Big Sur style
    rounded_rect(mask_draw, [PADDING//2, PADDING//2, SIZE - PADDING//2, SIZE - PADDING//2], corner_radius, 255)

    img.paste(grad_rgba, (0, 0), mask)
    draw = ImageDraw.Draw(img)

    # --- Clipboard body ---
    # Main clipboard rectangle (slightly off-white)
    clip_left = 240
    clip_right = 784
    clip_top = 260
    clip_bottom = 850
    clip_radius = 32

    # Shadow
    shadow = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow)
    rounded_rect(shadow_draw, [clip_left + 6, clip_top + 8, clip_right + 6, clip_bottom + 8], clip_radius, (0, 0, 0, 60))
    img = Image.alpha_composite(img, shadow)
    draw = ImageDraw.Draw(img)

    # Clipboard body - warm white
    rounded_rect(draw, [clip_left, clip_top, clip_right, clip_bottom], clip_radius, (248, 248, 252, 255))

    # --- Clipboard clip (top) ---
    clip_width = 200
    clip_height = 60
    clip_cx = (clip_left + clip_right) // 2
    clip_clip_left = clip_cx - clip_width // 2
    clip_clip_right = clip_cx + clip_width // 2
    clip_clip_top = clip_top - 30
    clip_clip_bottom = clip_top + 30

    # Clip rectangle (darker gray/silver)
    rounded_rect(draw, [clip_clip_left, clip_clip_top, clip_clip_right, clip_clip_bottom], 14, (160, 168, 180, 255))

    # Inner clip hole
    hole_w = 80
    hole_h = 28
    rounded_rect(draw, [clip_cx - hole_w//2, clip_clip_top + 8, clip_cx + hole_w//2, clip_clip_top + 8 + hole_h], 10, (120, 128, 140, 255))

    # --- Text lines on clipboard ---
    line_color = (180, 190, 210, 255)
    line_y_start = 370
    line_spacing = 52
    line_left = clip_left + 60
    line_right_full = clip_right - 60

    line_lengths = [1.0, 0.75, 0.9, 0.6, 0.85, 0.5, 0.7]

    for i, length in enumerate(line_lengths):
        y = line_y_start + i * line_spacing
        if y + 14 > clip_bottom - 40:
            break
        right = line_left + int((line_right_full - line_left) * length)
        rounded_rect(draw, [line_left, y, right, y + 14], 7, line_color)

    # --- Checkmark/shield accent (bottom-right) ---
    # Small shield shape to represent security/vault
    shield_cx = clip_right - 30
    shield_cy = clip_bottom - 30
    shield_size = 100

    # Shield background circle
    circle_color = (41, 182, 115, 255)  # Green accent
    draw.ellipse([shield_cx - shield_size//2, shield_cy - shield_size//2,
                  shield_cx + shield_size//2, shield_cy + shield_size//2], fill=circle_color)

    # White checkmark
    check_pts = [
        (shield_cx - 30, shield_cy - 2),
        (shield_cx - 10, shield_cy + 22),
        (shield_cx + 32, shield_cy - 24),
    ]
    draw.line([check_pts[0], check_pts[1]], fill=(255, 255, 255, 255), width=14)
    draw.line([check_pts[1], check_pts[2]], fill=(255, 255, 255, 255), width=14)
    # Round the joints
    for pt in check_pts:
        draw.ellipse([pt[0]-7, pt[1]-7, pt[0]+7, pt[1]+7], fill=(255, 255, 255, 255))

    # --- Subtle highlight on top-left of background ---
    highlight = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    h_draw = ImageDraw.Draw(highlight)
    for i in range(80):
        alpha = int(30 * (1 - i / 80))
        h_draw.ellipse([PADDING//2 - 100 + i, PADDING//2 - 100 + i,
                        PADDING//2 + 400 - i, PADDING//2 + 400 - i],
                       fill=(255, 255, 255, alpha))

    img = Image.alpha_composite(img, highlight)

    return img


def save_iconset(img, output_dir):
    """Save image as iconset directory with all required sizes."""
    iconset_dir = os.path.join(output_dir, 'AppIcon.iconset')
    os.makedirs(iconset_dir, exist_ok=True)

    sizes = [
        ('icon_16x16.png', 16),
        ('icon_16x16@2x.png', 32),
        ('icon_32x32.png', 32),
        ('icon_32x32@2x.png', 64),
        ('icon_128x128.png', 128),
        ('icon_128x128@2x.png', 256),
        ('icon_256x256.png', 256),
        ('icon_256x256@2x.png', 512),
        ('icon_512x512.png', 512),
        ('icon_512x512@2x.png', 1024),
    ]

    for filename, size in sizes:
        resized = img.resize((size, size), Image.LANCZOS)
        resized.save(os.path.join(iconset_dir, filename))
        print(f'  Created {filename} ({size}x{size})')

    return iconset_dir


if __name__ == '__main__':
    script_dir = os.path.dirname(os.path.abspath(__file__))

    print('Generating ClipVault icon (1024x1024)...')
    icon = generate_icon()

    # Save source PNG
    source_path = os.path.join(script_dir, 'icon-1024.png')
    icon.save(source_path)
    print(f'Saved source: {source_path}')

    # Save iconset
    print('Creating iconset...')
    iconset_dir = save_iconset(icon, script_dir)
    print(f'Iconset: {iconset_dir}')

    # Convert to .icns using iconutil
    icns_path = os.path.join(script_dir, 'AppIcon.icns')
    print('Converting to .icns...')
    os.system(f'iconutil -c icns "{iconset_dir}" -o "{icns_path}"')

    if os.path.exists(icns_path):
        size_kb = os.path.getsize(icns_path) // 1024
        print(f'Created: {icns_path} ({size_kb} KB)')
    else:
        print('ERROR: iconutil failed to create .icns file')

    print('Done!')
