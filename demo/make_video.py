"""Render the 30-second demo video (demo/anchor-demo.mp4, 3840x2160, 30 fps).

Story: what the 2025 Global Winners' demos share (studied 14 Nov 2026) - a concrete stake first,
the product in motion, evidence, a closing card; one idea per scene, narrated. Every app shot is a
4K frame-by-frame capture of the running app (demo/record.mjs); a virtual camera eases into the
part that matters so interface text reads at full size. Cards are typeset in the app's own font.
Voice: Kokoro (open-weights neural TTS, voice af_heart), disclosed on the end card and in metadata.
The background tone is synthesised here; no third-party audio or imagery is used.

Usage: python demo/make_video.py <4K recordings dir> <work dir>
<work dir> holds fonts/atk-{Regular,Bold}.ttf and kokoro/{kokoro-v1.0.onnx,voices-v1.0.bin}.
"""
import pathlib
import subprocess
import sys
import wave

import numpy as np
from PIL import Image, ImageDraw, ImageFont

REC, WORK = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
OUT = pathlib.Path(__file__).with_name("anchor-demo.mp4")
W, H, FPS, SR = 3840, 2160, 30, 48000
AMBER, TEXT, MUTED, GROUND = (255, 184, 88), (255, 226, 189), (211, 149, 72), (13, 9, 4)
LEAD, GAP, LIMIT = 0.25, 0.28, 29.95
BAND = 300  # subtitle band height at 4K
FONT = {b: str(WORK / "fonts" / f"atk-{'Bold' if b else 'Regular'}.ttf") for b in (True, False)}
_fonts: dict = {}


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    return _fonts.setdefault((size, bold), ImageFont.truetype(FONT[bold], size))


def ease(t: float) -> float:
    t = min(1.0, max(0.0, t))
    return 1 - (1 - t) ** 3


def text(img, s, size, y, color, bold=False, t=1.0, t0=0.0, cx=W / 2, track=0):
    """Centred text that fades and rises in from t0 over 0.45 s."""
    e = ease((t - t0) / 0.45)
    if e <= 0:
        return
    f = font(size, bold)
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    dy = 40 * (1 - e)
    if track:
        widths = [d.textlength(ch, font=f) + track for ch in s]
        x = cx - (sum(widths) - track) / 2
        for ch, w in zip(s, widths):
            d.text((x, y + dy), ch, font=f, fill=(*color, int(255 * e)))
            x += w
    else:
        d.text((cx - d.textlength(s, font=f) / 2, y + dy), s, font=f, fill=(*color, int(255 * e)))
    img.alpha_composite(layer)


def background() -> Image.Image:
    yy, xx = np.mgrid[0:H, 0:W]
    r = np.sqrt(((xx - W / 2) / (W * 0.62)) ** 2 + ((yy - H * 0.45) / (H * 0.75)) ** 2)
    k = np.clip(1 - r, 0, 1) ** 1.6
    base = np.array(GROUND, float) + k[..., None] * np.array([22, 14, 6], float)
    return Image.fromarray(base.astype(np.uint8)).convert("RGBA")


BG: Image.Image


def subtitle(img, s):
    img.alpha_composite(Image.new("RGBA", (W, BAND), (*GROUND, 236)), (0, H - BAND))
    f = font(78)
    d = ImageDraw.Draw(img)
    d.text(((W - d.textlength(s, font=f)) / 2, H - BAND / 2 - 50), s, font=f, fill=TEXT)


# ---------------- scenes ----------------

def hook(t):
    img = BG.copy()
    text(img, "TEAM ANCHOR   ·   NASA SPACE APPS 2026   ·   HEALTH MONITORING FOR ASTRONAUTS", 54, 290, MUTED, True, t, 0.0, track=6)
    text(img, "Up to 22 minutes away", 210, 560, AMBER, True, t, 0.1)
    # Earth to Mars: the light-time line draws itself, a signal pulse travels along it.
    d = ImageDraw.Draw(img)
    x0, x1, y = 760, 3080, 1260
    p = ease((t - 0.45) / 1.6)
    for x in range(x0, int(x0 + (x1 - x0) * p), 60):
        d.line([(x, y), (min(x + 34, x1), y)], fill=(*AMBER, 255), width=8)
    for x, name in ((x0, "Earth"), (x1, "Mars")):
        d.ellipse([x - 34, y - 34, x + 34, y + 34], outline=(*TEXT, 255), width=8, fill=(*GROUND, 255))
        f = font(64, True)
        d.text((x - d.textlength(name, font=f) / 2, y + 70), name, font=f, fill=TEXT)
    if 0 < p < 1:
        px = x0 + (x1 - x0) * p
        d.ellipse([px - 22, y - 22, px + 22, y + 22], fill=(*AMBER, 255))
    text(img, "each way, at the speed of light", 64, 1030, MUTED, False, t, 1.3)
    return img


def risk(t):
    img = BG.copy()
    text(img, "Out there, saying how you feel", 170, 700, AMBER, True, t, 0.0)
    text(img, "can feel risky.", 170, 930, AMBER, True, t, 0.35)
    return img


def evidence(t):
    img = BG.copy()
    text(img, "TESTED ON 71 REAL PEOPLE   ·   LIFESNAPS DATASET   ·   CC BY 4.0", 54, 300, MUTED, True, t, 0.0, track=6)
    for cx, target, label, t0 in ((W * 0.31, 79, "of mood days", 0.2), (W * 0.69, 78, "of sleep days", 0.45)):
        n = round(target * ease((t - t0) / 1.2))
        text(img, f"{n}%", 400, 620, AMBER, True, t, t0, cx=cx)
        text(img, label, 80, 1140, TEXT, False, t, t0 + 0.2, cx=cx)
    text(img, "inside the personal range built to hold 80% of days", 72, 1330, TEXT, False, t, 1.0)
    text(img, "and 2 to 5 times fewer notes than a naive single-day rule", 64, 1450, MUTED, False, t, 1.5)
    return img


def end(t):
    img = BG.copy()
    text(img, "ANCHOR", 320, 560, AMBER, True, t, 0.0, track=40)
    text(img, "A personal baseline for long missions", 96, 1000, TEXT, False, t, 0.2)
    text(img, "Team ANCHOR   ·   NASA Space Apps 2026", 68, 1170, MUTED, False, t, 0.35)
    text(img, "agcpr.github.io/nasa-space-apps-2026-anchor", 68, 1270, TEXT, True, t, 0.45)
    text(img, "Synthetic demo data   ·   Software built with AI assistance (Claude)   ·   Synthetic voice (Kokoro TTS)", 44, 1660, MUTED, False, t, 0.6)
    return img


def app_scene(name, keys):
    """A recording under an eased virtual camera; keys = [(time, (x, y, size))] in source fractions."""
    frames = sorted((REC / name).glob("f*.jpg"))

    def render(t, dur):
        n = len(frames)
        speed = max(1.0, n / (dur * FPS))
        src = Image.open(frames[min(n - 1, int(t * FPS * speed))])
        a = b = keys[-1][1]
        ta, tb = keys[-1][0], keys[-1][0] + 1
        for (t1, k1), (t2, k2) in zip(keys, keys[1:]):
            if t <= t2:
                (ta, a), (tb, b) = (t1, k1), (t2, k2)
                break
        e = ease((t - ta) / max(1e-6, tb - ta))
        x, y, s = (a[i] + (b[i] - a[i]) * e for i in range(3))
        box = (x * src.width, y * src.height, (x + s) * src.width, (y + s) * src.height)
        return src.resize((W, H), Image.LANCZOS, box=box).convert("RGBA")

    return render


LINES = [
    ("Mars is up to twenty-two minutes away. Each way.", lambda t, d: hook(t)),
    ("Out there, saying how you feel can feel risky.", lambda t, d: risk(t)),
    ("ANCHOR is a ninety-second check-in. You rate first; your data stays yours.",
     app_scene("s3_checkin", [(0.0, (0, 0, 1)), (0.25, (0, 0, 1)), (1.2, (0.235, 0.03, 0.75))])),
    ("It learns your normal before flight, and notices lasting change, calmly.",
     app_scene("s4_course", [(0.0, (0.235, 0.08, 0.765)), (4.0, (0.25, 0.10, 0.75))])),
    ("Every note explains why, grounded in NASA research.",
     app_scene("s5_why", [(0.0, (0.235, 0.03, 0.75)), (3.0, (0.15, 0.11, 0.85))])),
    ("Nothing leaves unless you see it first. Safety lines always get through.",
     app_scene("s6_sharing", [(0.0, (0.235, 0.04, 0.75)), (1.4, (0.235, 0.04, 0.75)), (3.6, (0.27, 0.14, 0.6))])),
    ("On seventy-one real people, its range held eight days in ten.", lambda t, d: evidence(t)),
    ("ANCHOR. Team ANCHOR.", lambda t, d: end(t)),
]


def voice():
    from kokoro_onnx import Kokoro
    k = Kokoro(str(WORK / "kokoro" / "kokoro-v1.0.onnx"), str(WORK / "kokoro" / "voices-v1.0.bin"))
    out = []
    for line, _ in LINES:
        s, sr = k.create(line, voice="af_heart", speed=1.0, lang="en-us")
        out.append(np.interp(np.arange(0, len(s) * SR / sr) * sr / SR, np.arange(len(s)), s).astype(np.float32))
    return out


def tone(n):
    """A quiet A-major pad: slow swells, soft top end, fades at both ends."""
    t = np.arange(n) / SR
    pad = sum(a * np.sin(2 * np.pi * f * t + ph) * (0.6 + 0.4 * np.sin(2 * np.pi * r * t + ph))
              for f, a, r, ph in ((110.0, 1.0, 0.07, 0.0), (164.81, 0.7, 0.05, 1.3), (220.0, 0.55, 0.09, 2.1),
                                  (277.18, 0.35, 0.06, 0.4), (329.63, 0.3, 0.08, 2.9)))
    env = np.minimum(1, t / 2.0) * np.minimum(1, (t[-1] - t) / 1.5)
    return (pad / 3.0 * env).astype(np.float32)


def main():
    global BG
    BG = background()
    vo = voice()
    durs = [len(v) / SR + GAP for v in vo]
    durs[-1] += 0.3
    total = LEAD + sum(durs)
    assert total <= LIMIT, f"{total:.2f} s is over 30 s"

    frames_dir = WORK / "frames4k"
    frames_dir.mkdir(exist_ok=True)
    for old in frames_dir.glob("*.jpg"):
        old.unlink()
    audio = np.zeros(int(total * SR) + SR, np.float32)
    i, start = 0, LEAD
    for j, ((line, scene), d, v) in enumerate(zip(LINES, durs, vo)):
        at = int((start + 0.08) * SR)
        audio[at: at + len(v)] += v
        lead = LEAD if j == 0 else 0.0
        nf = round((d + lead) * FPS)
        for k in range(nf):
            img = scene(max(0.0, k / FPS - lead), d)
            fade = min(1.0, (k + 1) / 6, (nf - k) / 6)  # 0.2 s dip through the night ground
            if fade < 1:
                img = Image.blend(Image.new("RGBA", img.size, (*GROUND, 255)), img, fade)
            subtitle(img, line)
            img.convert("RGB").save(frames_dir / f"{i:05d}.jpg", quality=94)
            i += 1
        start += d
        print(f"scene {j}: {d:.2f} s")

    audio = audio[: int(total * SR)]
    mix = audio + 0.035 * tone(len(audio))
    mix = mix / max(1e-6, float(np.abs(mix).max())) * 0.89
    wav = WORK / "mix.wav"
    with wave.open(str(wav), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes((mix * 32767).astype(np.int16).tobytes())

    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-framerate", str(FPS), "-i", str(frames_dir / "%05d.jpg"), "-i", str(wav),
                    "-c:v", "libx264", "-preset", "slow", "-crf", "19", "-profile:v", "high", "-pix_fmt", "yuv420p",
                    "-c:a", "aac", "-b:a", "192k", "-shortest", "-movflags", "+faststart",
                    "-metadata", "title=ANCHOR: NASA Space Apps 2026 demo (Team ANCHOR)",
                    "-metadata", "comment=Real app captures with synthetic demo data. Software built with AI assistance (Claude). Narration is synthetic speech (Kokoro TTS, voice af_heart).",
                    str(OUT)], check=True)
    print(f"{OUT}: {total:.2f} s, {i} frames")


if __name__ == "__main__":
    main()
