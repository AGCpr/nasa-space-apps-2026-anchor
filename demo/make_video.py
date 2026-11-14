"""Build the 30-second demo video from real captures of the running app (no generated imagery).

Needs: Chrome (headless), ffmpeg, and the app served at http://localhost:8765/app/ from the repo root
(`python -m http.server 8765`). Output: demo/anchor-demo.mp4 (1920x1080, 30 s, English captions).
Usage: python -I demo/make_video.py
"""
import pathlib
import subprocess
import tempfile

CHROME = r"C:\Program Files\Google\Chrome\Application\chrome.exe"
URL = "http://localhost:8765/app/#"
FONT = "C\\:/Windows/Fonts/segoeui.ttf"
BOLD = "C\\:/Windows/Fonts/segoeuib.ttf"
OUT = pathlib.Path(__file__).with_name("anchor-demo.mp4")
GROUND, TEXT, MUTED = "0x0d0904", "0xffb858", "0xd39548"
FOOT = "Synthetic demo data, not astronaut data   |   Software built with AI assistance (Claude)"

# (seconds, app hash or None for a text card, caption lines)
SCENES = [
    (3.5, None, ["ANCHOR", "A personal baseline for astronaut wellbeing",
                 "Team ANCHOR  |  NASA Space Apps 2026  |  Create Health Monitoring Software for Astronauts"]),
    (4.5, "view=today&day=125&light=evening",
     ["Rate first, then see tonight inside your own usual range.", "A 90-second evening check-in, private by default."]),
    (5.0, "view=course&day=150&light=evening",
     ["Your own normal, learned before flight. Sustained change is noticed calmly:",
      "at most one unwanted note per 30 days. \"This is my new normal\" is your choice, and reversible."]),
    (4.5, "view=today&day=110&light=day&why=open",
     ["Every note explains itself: the data, the rule, what closes it,", "and the NASA Human Research Program area it relates to."]),
    (4.5, "view=sharing&day=150&light=evening",
     ["Nothing leaves the tablet unless it is listed first.", "Pre-agreed safety lines still always work, even during loss of signal."]),
    (4.5, None, ["Tested on 71 real people (LifeSnaps, CC BY 4.0)",
                 "The personal range held 79% of mood days and 78% of sleep days (target 80%).",
                 "Notes came 2 to 5 times less often than a naive single-day rule."]),
    (3.5, None, ["Next: an analog mission trial.", "A companion astronauts want to use, built for the delay to Mars.",
                 "github.com/AGCpr/nasa-space-apps-2026-anchor"]),
]


def capture(tmp: pathlib.Path, i: int, frag: str) -> pathlib.Path:
    png = tmp / f"s{i}.png"
    subprocess.run([CHROME, "--headless=new", "--disable-gpu", "--hide-scrollbars", "--force-device-scale-factor=1",
                    "--window-size=1280,800", "--virtual-time-budget=20000", f"--screenshot={png}", URL + frag],
                   check=True, capture_output=True)
    return png


def text_filters(tmp: pathlib.Path, i: int, lines: list[str], card: bool) -> str:
    parts = []
    for j, line in enumerate(lines):
        f = tmp / f"t{i}_{j}.txt"
        f.write_text(line, encoding="utf-8")
        tf = str(f).replace("\\", "/").replace(":", "\\:")
        if card:
            title = i == 0
            size = (96 if title else 56) if j == 0 else 40 if j == 1 else 32
            y = [360, 500, 570][j] if title else [400, 500, 560][j]
            font, color = (BOLD, TEXT) if j == 0 else (FONT, TEXT if j == 1 else MUTED)
        else:
            size, y, font, color = 32, 940 + 44 * j + (20 if len(lines) == 1 else 0), FONT, TEXT
        parts.append(f"drawtext=expansion=none:fontfile='{font}':textfile='{tf}':fontsize={size}:fontcolor={color}:x=(w-text_w)/2:y={y}")
    foot = tmp / "foot.txt"
    foot.write_text(FOOT, encoding="utf-8")
    ff = str(foot).replace("\\", "/").replace(":", "\\:")
    parts.append(f"drawtext=expansion=none:fontfile='{FONT}':textfile='{ff}':fontsize=22:fontcolor={MUTED}:x=(w-text_w)/2:y=h-30")
    return ",".join(parts)


def main() -> None:
    with tempfile.TemporaryDirectory() as d:
        tmp = pathlib.Path(d)
        clips = []
        for i, (secs, frag, lines) in enumerate(SCENES):
            clip = tmp / f"c{i}.mp4"
            fade = f"fade=t=in:st=0:d=0.35,fade=t=out:st={secs - 0.35}:d=0.35"
            if frag is None:
                src = ["-f", "lavfi", "-i", f"color=c={GROUND}:s=1920x1080:d={secs}:r=30"]
                vf = f"{text_filters(tmp, i, lines, True)},{fade}"
            else:
                png = capture(tmp, i, frag)
                src = ["-loop", "1", "-t", str(secs), "-i", str(png)]
                # The 1280x800 tablet capture, scaled to 1474x921 and centred above a caption band.
                vf = (f"scale=1474:921,pad=1920:1080:223:8:color={GROUND},"
                      f"{text_filters(tmp, i, lines, False)},{fade}")
            subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *src, "-vf", vf, "-r", "30", "-pix_fmt", "yuv420p",
                            "-c:v", "libx264", "-t", str(secs), str(clip)], check=True)
            clips.append(clip)
        lst = tmp / "list.txt"
        lst.write_text("".join(f"file '{c.as_posix()}'\n" for c in clips), encoding="utf-8")
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(lst),
                        "-c", "copy", "-movflags", "+faststart", str(OUT)], check=True)
    total = sum(s for s, _, _ in SCENES)
    print(f"{OUT} ({total:.1f} s)")
    assert total <= 30.0, "Space Apps demo videos must be 30 seconds or less"


if __name__ == "__main__":
    main()
