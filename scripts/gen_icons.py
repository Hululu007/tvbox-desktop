#!/usr/bin/env python3
"""生成 TVBox Desktop 全平台图标：png / ico / icns"""
from PIL import Image, ImageDraw, ImageFont
import struct, os

OUT = "/home/z/my-project/tvbox-desktop-tauri/src-tauri/icons"
os.makedirs(OUT, exist_ok=True)

S = 1024
img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(img)

# 背景：蓝色圆角方块（Tauri 图标不需要系统自动圆角，手动留边）
r = int(S * 0.22)
m = int(S * 0.06)
# 竖向渐变
grad = Image.new("RGBA", (S, S), (0, 0, 0, 0))
gd = ImageDraw.Draw(grad)
top = (37, 99, 235)      # #2563EB
bottom = (29, 78, 216)   # #1D4ED8
for y in range(m, S - m):
    t = (y - m) / max(1, (S - 2 * m))
    c = tuple(int(top[i] + (bottom[i] - top[i]) * t) for i in range(3)) + (255,)
    gd.line([(m, y), (S - m, y)], fill=c)
mask = Image.new("L", (S, S), 0)
md = ImageDraw.Draw(mask)
md.rounded_rectangle([m, m, S - m, S - m], radius=r, fill=255)
img.paste(grad, (0, 0), mask)

# 白色 "TV" 文字
def load_font(size):
    for p in [
        "/usr/share/fonts/truetype/chinese/NotoSansSC-Bold.ttf",
        "/usr/share/fonts/truetype/chinese/NotoSansSC-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    ]:
        if os.path.exists(p):
            return ImageFont.truetype(p, size)
    return ImageFont.load_default()

font = load_font(int(S * 0.42))
text = "TV"
bbox = d.textbbox((0, 0), text, font=font)
tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
tx = (S - tw) // 2 - bbox[0]
ty = (S - th) // 2 - bbox[3] + int(S * 0.01)
d.text((tx, ty), text, font=font, fill=(255, 255, 255, 255))

# 右上小圆点（极简装饰）
dot_r = int(S * 0.055)
d.ellipse(
    [S - m - dot_r * 3 - int(S * 0.05), m + int(S * 0.055),
     S - m - int(S * 0.05), m + int(S * 0.055) + dot_r * 2],
    fill=(147, 197, 253, 255),
)

# 导出 png
img.resize((512, 512), Image.LANCZOS).save(os.path.join(OUT, "icon.png"))
img.resize((256, 256), Image.LANCZOS).save(os.path.join(OUT, "128x128@2x.png"))
img.resize((128, 128), Image.LANCZOS).save(os.path.join(OUT, "128x128.png"))
img.resize((32, 32), Image.LANCZOS).save(os.path.join(OUT, "32x32.png"))
img.resize((64, 64), Image.LANCZOS).save(os.path.join(OUT, "64x64.png"))

# ico：多尺寸
img.save(os.path.join(OUT, "icon.ico"),
         sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])

# icns：手工拼装 PNG chunk 容器（ic07/ic08/ic09/ic10）
def icns_from_png(png_bytes: bytes, ctype: bytes) -> bytes:
    return ctype + struct.pack(">I", len(png_bytes) + 8) + png_bytes

body = b""
import io
for ctype, size in [
    (b"ic07", 128), (b"ic08", 256), (b"ic09", 512), (b"ic10", 1024),
]:
    im = img.resize((size, size), Image.LANCZOS)
    bio = io.BytesIO()
    im.save(bio, format="PNG")
    body += icns_from_png(bio.getvalue(), ctype)

icns = b"icns" + struct.pack(">I", len(body) + 8) + body
with open(os.path.join(OUT, "icon.icns"), "wb") as f:
    f.write(icns)

print("icons OK:", os.listdir(OUT))
