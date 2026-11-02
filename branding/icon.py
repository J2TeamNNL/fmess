"""fMess icon: a white f on the gradient of fmess.app's feature icons (pink to
purple to blue, corner to corner), in Apple's rounded square, drawn per size:
a shadow when large, a bigger f and less margin when small.

    python3 branding/icon.py OUT_DIR   # icon-16.svg ... icon-1024.svg

Render each SVG at its own size (Chrome headless keeps the transparency:
--headless=new --force-device-scale-factor=1 --default-background-color=00000000
--window-size=N,N --screenshot=icon-N.png), copy icon-1024.png to app-icon.png
and run `npx tauri icon app-icon.png -o <dir>` for icon.ico and the Square*/Store
logos, then replace icon.icns (iconutil -c icns on an .iconset of the per-size
PNGs) and 32x32, 64x64, 128x128, 128x128@2x and icon.png (512) with the per-size
renders; the site's favicons, icon-256/512 and mark.png (128) too, then og.png
from og.html.

    python3 branding/icon.py --themes OUT_DIR   # zalo-256.svg ... teams-512.svg

The icons the app shows while it runs with the Zalo, WhatsApp or Teams theme (see
icon.rs): that app's own logo (brands/: Zalo's and WhatsApp's public domain on
Wikimedia Commons, Teams' as Teams draws it) with fMess's icon as a badge in the
corner, at 512 for the Dock and 256 for the window and tray. Render them as above
into src-tauri/icons/themes/."""
import math, pathlib, re, sys

def superellipse(cx, cy, w, h, n, steps=720):
    pts = []
    for i in range(steps):
        t = 2 * math.pi * i / steps
        c, s = math.cos(t), math.sin(t)
        pts.append(f"{cx + (w/2) * math.copysign(abs(c) ** (2/n), c):.2f},{cy + (h/2) * math.copysign(abs(s) ** (2/n), s):.2f}")
    return "M" + " L".join(pts) + " Z"

SQUIRCLE = superellipse(512, 512, 824, 824, 5.0)

F_STEM = "M-21,154 V-18 C-21,-74 9,-100 61,-100 H90"
F_BAR = "M-89,10 H51"
# The gradient of the site's feature icons, top left to bottom right.
INK = [("0", "#FF5FAE"), ("0.5", "#A65BFF"), ("1", "#4C9CFF")]
INK_FROM, INK_TO = (180, 150), (850, 880)

def gradient(id_, a, b):
    stops = "".join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in INK)
    return f'<linearGradient id="{id_}" gradientUnits="userSpaceOnUse" x1="{a[0]:.1f}" y1="{a[1]:.1f}" x2="{b[0]:.1f}" y2="{b[1]:.1f}">{stops}</linearGradient>'

def svg(size):
    small = size <= 32
    square = superellipse(512, 512, 920, 920, 5.0) if small else SQUIRCLE
    scale, stem, bar = (1.7, 66, 60) if small else (1.55, 62, 56)
    shadow = "" if small else f'''<filter id="iconshadow" filterUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024"><feDropShadow dx="0" dy="8" stdDeviation="{8 if size >= 256 else 5}" flood-color="#000" flood-opacity="0.3"/></filter>'''
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 1024 1024">
<defs>
  {gradient("ink", INK_FROM, INK_TO)}
  <radialGradient id="toplight" cx="0.5" cy="-0.1" r="0.8"><stop offset="0" stop-color="#FFFFFF" stop-opacity="0.18"/><stop offset="1" stop-color="#FFFFFF" stop-opacity="0"/></radialGradient>
  <clipPath id="body"><path d="{square}"/></clipPath>
  {shadow}
</defs>
<g {'' if small else 'filter="url(#iconshadow)"'}><path d="{square}" fill="url(#ink)"/></g>
<g clip-path="url(#body)"><rect width="1024" height="1024" fill="url(#toplight)"/></g>
<g transform="translate(512,512) scale({scale})" fill="none" stroke="#FFFFFF" stroke-linecap="round" stroke-linejoin="round">
  <path d="{F_STEM}" stroke-width="{stem}"/><path d="{F_BAR}" stroke-width="{bar}"/>
</g>
</svg>'''

BRANDS = pathlib.Path(__file__).parent / "brands"

def inner(svg_text):
    """What is inside an SVG's outer <svg> element."""
    return svg_text[svg_text.index(">", svg_text.index("<svg")) + 1:svg_text.rindex("</svg>")]

def brand(name):
    """The brand's logo, drawn in the icon's 1024 square like fMess's own."""
    logo = (BRANDS / f"{name}.svg").read_text()
    if name == "zalo":
        # Zalo's logo is its whole app icon, a rounded square (50 units).
        return f'<g filter="url(#logoshadow)"><g transform="translate(100,100) scale({824 / 50})">{inner(logo)}</g></g>'
    if name == "teams":
        # Teams' logo alone (20 units from 2,2), on a white square as its app shows it.
        return f'''<g filter="url(#logoshadow)"><path d="{SQUIRCLE}" fill="#ffffff"/></g>
<g transform="translate(176,176) scale(28)">{inner(logo)}</g>'''
    # WhatsApp's is the bubble alone: on a green square, as its app shows it,
    # without the logo's grey shadow (filter a) or the path drawn under the
    # same one again (its gradient isn't in the file).
    paths = [p for p in re.findall(r"<path[^>]*/>", logo) if 'filter="url(#a)"' not in p and "linearGradient1780" not in p]
    defs = re.search(r"<defs>.*?</defs>", logo).group(0)
    return f'''<g filter="url(#logoshadow)"><path d="{SQUIRCLE}" fill="url(#wa)"/></g>
{defs}<g transform="translate({512 - 87.2 * 3.6:.1f},{512 - 87.2 * 3.6:.1f}) scale(3.6)">{"".join(paths)}</g>'''

def themed(size, name):
    """A brand's logo with fMess's icon as a badge on its bottom right corner."""
    badge = inner(svg(64))
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 1024 1024">
<defs>
  <filter id="logoshadow" filterUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024"><feDropShadow dx="0" dy="8" stdDeviation="8" flood-color="#000" flood-opacity="0.35"/></filter>
  <linearGradient id="wa" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#5FE07A"/><stop offset="1" stop-color="#1FAF4B"/></linearGradient>
</defs>
{brand(name)}
<g transform="translate(600,600) scale(0.41)">{badge}</g>
</svg>'''

if __name__ == "__main__":
    if sys.argv[1] == "--themes":
        out = pathlib.Path(sys.argv[2])
        for name in ("zalo", "whatsapp", "teams"):
            for size in (256, 512):
                (out / f"{name}-{size}.svg").write_text(themed(size, name))
        sys.exit()
    out = pathlib.Path(sys.argv[1])
    for size in (16, 32, 64, 128, 256, 512, 1024):
        (out / f"icon-{size}.svg").write_text(svg(size))
