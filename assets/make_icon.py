"""Draws the TV Hub icon.

Writes icon.ico (embedded into the .exe by build.rs), icon-64.rgba (raw RGBA,
the window icon at runtime) and icon-256.png (the in-app logo). Run from this
directory:
    python make_icon.py
"""
from PIL import Image, ImageDraw

SS = 4  # supersampling for anti-aliased edges
TOP = (0xFF, 0x5A, 0x4E)
BOTTOM = (0xB3, 0x1B, 0x52)
WHITE = (0xFF, 0xFF, 0xFF, 255)


def render(size):
    s = size * SS
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))

    # Vertical gradient clipped to a rounded square.
    grad = Image.new("RGBA", (s, s))
    gd = ImageDraw.Draw(grad)
    for y in range(s):
        t = y / (s - 1)
        c = tuple(round(a + (b - a) * t) for a, b in zip(TOP, BOTTOM))
        gd.line([(0, y), (s, y)], fill=c + (255,))
    mask = Image.new("L", (s, s), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, s - 1, s - 1], radius=int(s * 0.22), fill=255)
    img.paste(grad, (0, 0), mask)

    d = ImageDraw.Draw(img)
    # Antenna.
    w = max(1, int(s * 0.045))
    d.line([(s * 0.38, s * 0.16), (s * 0.5, s * 0.3)], fill=WHITE, width=w)
    d.line([(s * 0.62, s * 0.16), (s * 0.5, s * 0.3)], fill=WHITE, width=w)
    # Screen outline.
    d.rounded_rectangle([s * 0.18, s * 0.3, s * 0.82, s * 0.78], radius=int(s * 0.08), outline=WHITE, width=int(s * 0.055))
    # Play triangle.
    d.polygon([(s * 0.44, s * 0.42), (s * 0.44, s * 0.66), (s * 0.63, s * 0.54)], fill=WHITE)
    return img.resize((size, size), Image.LANCZOS)


if __name__ == "__main__":
    big = render(256)
    big.save("icon-256.png")
    big.save("icon.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    with open("icon-64.rgba", "wb") as f:
        f.write(render(64).tobytes())
