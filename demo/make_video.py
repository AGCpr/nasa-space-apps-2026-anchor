"""Compose the 30-second demo video (demo/anchor-demo.mp4).

Structure follows what the 2025 Global Winners' demos share (studied 14 Nov 2026): a concrete stake
in the first seconds, the working product in motion, evidence, a closing card; one idea per scene,
narrated, with large type. Every app shot is a real capture of the running app (demo/record.mjs);
text cards are typeset here. Voice: Microsoft Edge neural TTS (synthetic, disclosed on the end card).

Usage (from the repository root, after `node demo/record.mjs` and encoding its frames to <rec>/<scene>.mp4):
  python demo/make_video.py <recordings dir> <work dir>
needs ffmpeg, the edge-tts package, and Atkinson Hyperlegible converted to TTF in <work dir>/fonts.
"""
import pathlib
import subprocess
import sys

REC = pathlib.Path(sys.argv[1])
WORK = pathlib.Path(sys.argv[2])
OUT = pathlib.Path(__file__).with_name("anchor-demo.mp4")
REG = (WORK / "fonts" / "atk-Regular.ttf").as_posix().replace(":", "\\:")
BOLD = (WORK / "fonts" / "atk-Bold.ttf").as_posix().replace(":", "\\:")
AMBER, TEXT, MUTED = "0xffb858", "0xffe2bd", "0xd39548"
VOICE, RATE = "en-US-AvaNeural", "+14%"
LIMIT = 30.0
# Height of the subtitle band; in app shots it replaces the replay toolbar strip (150 px at 1.5x).
BAND = 150

# (voice line, scene): a scene is ("card", [text items]) or ("app", recording name).
# A text item is (text, size, y, colour, bold, appear-at seconds[, x expression]); x is centred by default.
SCENES = [
    ("On the way to Mars, help is up to twenty-two minutes away.",
     ("card", [("TEAM ANCHOR  ·  NASA SPACE APPS 2026  ·  HEALTH MONITORING FOR ASTRONAUTS", 26, 150, MUTED, True, 0.0),
               ("Up to 22 minutes", 132, 360, AMBER, True, 0.15),
               ("between a Mars crew and help from Earth. Each way.", 46, 540, TEXT, False, 0.7)])),
    ("Crews can under-report when monitoring feels like surveillance.",
     ("card", [("Crews can under-report", 112, 380, AMBER, True, 0.0),
               ("when monitoring feels like surveillance.", 46, 540, TEXT, False, 0.5)])),
    ("ANCHOR is a ninety-second check-in. You rate first; your data stays yours.", ("app", "s3_checkin")),
    ("It learns your normal before flight, and notices sustained change, calmly.", ("app", "s4_course")),
    ("Every note explains itself, grounded in NASA research.", ("app", "s5_why")),
    ("Nothing leaves unless you see it first. Safety lines always get through.", ("app", "s6_sharing")),
    ("On seventy-one real people, its range held eight days in ten.",
     ("card", [("TESTED ON REAL PEOPLE  ·  LIFESNAPS  ·  71 ADULTS  ·  CC BY 4.0", 26, 170, MUTED, True, 0.0),
               ("79%", 170, 330, AMBER, True, 0.2, "(w/2-text_w)/2+60"),
               ("78%", 170, 330, AMBER, True, 0.45, "w/2+(w/2-text_w)/2-60"),
               ("of mood days in the 80% range", 34, 540, TEXT, False, 0.35, "(w/2-text_w)/2+60"),
               ("of sleep days in the 80% range", 34, 540, TEXT, False, 0.6, "w/2+(w/2-text_w)/2-60"),
               ("and 2 to 5 times fewer notes than a naive single-day rule", 38, 680, TEXT, False, 1.1)])),
    ("ANCHOR. Team ANCHOR.",
     ("card", [("ANCHOR", 150, 330, AMBER, True, 0.0),
               ("A personal baseline for long missions", 46, 520, TEXT, False, 0.25),
               ("Team ANCHOR  ·  NASA Space Apps 2026", 34, 610, MUTED, False, 0.4),
               ("agcpr.github.io/nasa-space-apps-2026-anchor", 34, 665, TEXT, True, 0.5),
               ("Synthetic demo data  ·  Software built with AI assistance (Claude)  ·  Synthetic voice (Microsoft neural TTS)", 24, 860, MUTED, False, 0.6)])),
]


def run(*args: str) -> None:
    subprocess.run(args, check=True)


def duration(p: pathlib.Path) -> float:
    return float(subprocess.check_output(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(p)]))


def tfile(text: str, name: str) -> str:
    f = WORK / "txt" / name
    f.parent.mkdir(exist_ok=True)
    f.write_text(text, encoding="utf-8")
    return f.as_posix().replace(":", "\\:")


def draw(text, size, y, color, bold, t0, x="(w-text_w)/2", key="") -> str:
    a = f"if(lt(t\\,{t0})\\,0\\,if(lt(t\\,{t0 + 0.35})\\,(t-{t0})/0.35\\,1))"
    yy = f"{y}+22*(1-min(1\\,max(0\\,(t-{t0})/0.35)))"
    return (f"drawtext=expansion=none:fontfile='{BOLD if bold else REG}':textfile='{tfile(text, key)}':"
            f"fontsize={size}:fontcolor={color}:alpha='{a}':x={x}:y='{yy}'")


def subtitle(text: str, key: str) -> str:
    return (f"drawbox=x=0:y=ih-{BAND}:w=iw:h={BAND}:color=0x0d0904@0.9:t=fill,"
            f"drawtext=expansion=none:fontfile='{REG}':textfile='{tfile(text, key)}':fontsize=38:fontcolor={TEXT}:x=(w-text_w)/2:y=h-{BAND // 2 + 22}")


def main() -> None:
    WORK.mkdir(exist_ok=True)
    # Voice first: each scene lasts as long as its line.
    lines = []
    for i, (line, _) in enumerate(SCENES):
        mp3 = WORK / f"vo{i}.mp3"
        run(sys.executable, "-m", "edge_tts", "--voice", VOICE, f"--rate={RATE}", "--text", line, "--write-media", str(mp3))
        lines.append((mp3, duration(mp3)))
    durs = [d for _, d in lines]
    durs[-1] += 0.25
    assert sum(durs) <= LIMIT, f"{sum(durs):.2f} s is over the 30-second limit"

    # Card background: the app's evening ground with a soft vignette (no imagery).
    bg = WORK / "bg.png"
    run("ffmpeg", "-y", "-loglevel", "error", "-f", "lavfi", "-i", "color=c=0x1a1209:s=1920x1080:d=1", "-frames:v", "1",
        "-vf", "vignette=angle=PI/3.2", str(bg))

    segs = []
    for i, ((line, (kind, spec)), d) in enumerate(zip(SCENES, durs)):
        seg = WORK / f"seg{i}.mp4"
        fade = f"fade=t=in:st=0:d=0.18,fade=t=out:st={d - 0.18:.3f}:d=0.18"
        if kind == "card":
            texts = ",".join(draw(*item[:6], *(item[6:] or ["(w-text_w)/2"]), key=f"c{i}_{j}.txt") for j, item in enumerate(spec))
            src = ["-loop", "1", "-t", f"{d:.3f}", "-i", str(bg)]
            vf = f"scale=1920:1080,{texts},{subtitle(line, f's{i}.txt')},{fade}"
        else:
            clip = REC / f"{spec}.mp4"
            c = duration(clip)
            # Fit the recording to its line: speed up when longer, hold the last frame when shorter.
            fit = f"setpts=PTS*{d / c:.4f}" if c > d else f"tpad=stop_mode=clone:stop_duration={d - c + 0.05:.3f}"
            src = ["-i", str(clip)]
            vf = f"{fit},scale=1920:1080,crop=1920:{1080 - BAND}:0:0,pad=1920:1080:0:0:color=0x0d0904,{subtitle(line, f's{i}.txt')},{fade}"
        run("ffmpeg", "-y", "-loglevel", "error", *src, "-vf", vf, "-r", "30", "-t", f"{d:.3f}", "-an",
            "-c:v", "libx264", "-crf", "17", "-pix_fmt", "yuv420p", str(seg))
        segs.append(seg)

    vlist = WORK / "v.txt"
    vlist.write_text("".join(f"file '{s.as_posix()}'\n" for s in segs), encoding="utf-8")
    run("ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(vlist), "-c", "copy", str(WORK / "video.mp4"))

    # Audio: each line padded to its scene, concatenated, levelled.
    inputs, pads = [], []
    for i, ((mp3, _), d) in enumerate(zip(lines, durs)):
        inputs += ["-i", str(mp3)]
        pads.append(f"[{i}:a]aresample=48000,apad,atrim=0:{d:.3f}[a{i}]")
    graph = ";".join(pads) + ";" + "".join(f"[a{i}]" for i in range(len(lines))) + f"concat=n={len(lines)}:v=0:a=1,loudnorm=I=-16:TP=-1.5[out]"
    run("ffmpeg", "-y", "-loglevel", "error", *inputs, "-filter_complex", graph, "-map", "[out]", "-c:a", "aac", "-b:a", "160k", str(WORK / "audio.m4a"))

    run("ffmpeg", "-y", "-loglevel", "error", "-i", str(WORK / "video.mp4"), "-i", str(WORK / "audio.m4a"), "-map", "0:v", "-map", "1:a",
        "-c:v", "copy", "-c:a", "copy", "-shortest", "-movflags", "+faststart",
        "-metadata", "title=ANCHOR: NASA Space Apps 2026 demo (Team ANCHOR)",
        "-metadata", "comment=Real app captures with synthetic demo data. Software built with AI assistance (Claude). Narration is synthetic speech (Microsoft Edge neural TTS).",
        str(OUT))
    total = duration(OUT)
    print(f"{OUT} {total:.2f} s")
    assert total <= LIMIT + 0.05


if __name__ == "__main__":
    main()
