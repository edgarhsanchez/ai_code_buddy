# AI Code Buddy Assets

This directory contains visual assets for the AI Code Buddy project.

## Repository Cards

### `repo-card.svg` (1280x640)
- **Purpose**: Detailed repository card for social media and marketing
- **Features**: 
  - Professional dark theme design
  - Code example with vulnerability detection
  - Real-time analysis results display
  - Installation command
  - Key feature highlights
  - OWASP branding

### `social-preview.svg` (1280x640) 
- **Purpose**: Simplified social media preview card
- **Features**:
  - Clean, readable design
  - Essential feature highlights
  - Security-focused branding
  - GitHub repository link
  - Terminal-style code example

## Usage

These cards are designed for:
- **GitHub**: Repository social preview image
- **Social Media**: Twitter, LinkedIn, etc.
- **Documentation**: README headers and presentations
- **Marketing**: Blog posts and articles

## Design Elements

- **Color Scheme**: Dark theme with security-focused accent colors
- **Typography**: Professional system fonts
- **Icons**: Security shield, AI brain, code symbols
- **Layout**: Split design showcasing both branding and functionality

## GitHub Social Preview Setup

To use the social preview card on GitHub:

1. Go to repository Settings → Social preview
2. Upload `social-preview.svg` or convert to PNG
3. GitHub will display this image when the repository is shared

## Dimensions

Both cards follow the standard social media preview dimensions:
- **Width**: 1280px
- **Height**: 640px  
- **Aspect Ratio**: 2:1 (Twitter/GitHub standard)

## File Formats

- **SVG**: Vector format for scalability and web use
- **PNG**: Recommended for GitHub social preview (better compatibility)

To convert SVG to PNG, use any online converter or tools like:
```bash
# Using Inkscape (if installed)
inkscape --export-type=png --export-dpi=300 social-preview.svg

# Using ImageMagick (if installed)
magick convert -density 300 social-preview.svg social-preview.png
```
