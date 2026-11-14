"""Systematic landscape coding of 2026 Space Apps astronaut-health repositories.

Input : analysis/landscape/raw.tsv  (full_name, created_at, stars, description, url)
Output: analysis/landscape/readmes/<owner>__<repo>.md, coded.csv, summary.txt

Coding is keyword-based on README + description: a reproducible first pass, not a
substitute for reading. Run: py -I analysis/landscape/code_corpus.py
"""
import csv
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
RD = os.path.join(HERE, "readmes")
os.makedirs(RD, exist_ok=True)

FEATURES = {
    "personal_baseline": r"baseline|own normal|individual(ised|ized)? (norm|threshold)",
    "rolling_baseline": r"rolling (baseline|window|mean|average)|moving average",
    "fixed_start_baseline": r"first \d+ (days|sols)|baseline (period|phase) of",
    "ml_anomaly": r"isolation forest|autoencoder|lstm|one-class svm|random forest|xgboost|neural network|anomaly detection model",
    "stat_change_detection": r"cusum|ewma|control chart|change[- ]?point|page-hinkley|sequential (test|analysis)",
    "false_alarm_quantified": r"false (alarm|alert|positive)s? (rate|per)|specificity|precision|recall|f1|auroc|\bauc\b",
    "llm_or_chatbot": r"\bllm\b|gpt|gemini|openai|llama|ollama|chatbot|mistral|claude|language model",
    "offline_first": r"offline|edge[- ]ai|local[- ]first|no internet|without (internet|connectivity)",
    "privacy_or_consent": r"privacy|consent|encrypt",
    "psych_mood": r"\bmood\b|stress|mental|psycholog|emotion|loneliness|behaviou?ral health|well-?being",
    "psych_intervention": r"\bcbt\b|breathing exercise|meditation|mindfulness|journal(ing)?|coping|counsel",
    "sleep": r"\bsleep",
    "circadian": r"circadian|light exposure|melatonin|chronotype|phase shift",
    "cognitive_pvt": r"\bpvt\b|vigilance|reaction time|cognitive test|cognition",
    "radiation": r"radiation|\bdose\b|msv|solar particle",
    "camera_face": r"facial|face (recognition|detection|emotion)|camera|computer vision|opencv|mediapipe",
    "voice": r"\bvoice\b|speech|microphone|whisper",
    "gamification": r"gamif|badge|streak|\bxp\b|leaderboard|reward points",
    "body_figure": r"body map|human body|body figure|anatomy|clickable body|body diagram",
    "comm_delay": r"communication delay|signal delay|light[- ]time|blackout|conjunction|earth[- ]independent|delayed (link|comm)",
    "ground_dashboard": r"mission control|flight surgeon (dashboard|view|portal)|ground (station|view|dashboard)|fleet",
    "nasa_open_data": r"osdr|genelab|\blsda\b|inspiration4|twins? study|nasa (open )?data|noaa|swpc",
    "digital_twin": r"digital twin",
    "tests": r"pytest|jest|vitest|unit tests?|test suite",
    "non_diagnostic": r"not a medical device|non-?diagnostic|not (provide )?medical (advice|diagnos)|decision[- ]support only",
    "integrity_safety": r"watchdog|fail-?safe|redundan|hash[- ]chain|audit (log|trail|chain)|tamper",
    "crew_interpersonal": r"crew cohesion|conflict|team dynamics|interpersonal|crew relation",
    "lifecycle": r"pre-?flight|post-?flight|readaptation|re-?adaptation|return to earth",
    "family": r"\bfamily\b|families",
    "language_framing": r"nocebo|non-?alarming|calm (language|tone)|fear|anxiety-?free|supportive (tone|language)",
    "wearable": r"wearable|smartwatch|fitbit|whoop|oura|apple watch|astroskin|bio-?monitor",
    "explainability": r"explainab|shap|\blime\b|why (this|the) alert|rationale",
}
RELEVANT = re.compile(r"astronaut|crew", re.I)
HEALTH = re.compile(r"health|vital|well-?being|mental|medical|physiolog", re.I)
OFF_TOPIC = re.compile(r"human activity recognition|\bhar\b|bas experiment|farm|crop|soil|wildfire|"
                       r"hot ?spot|spherex|jukebox|lunar landing|mission design game|analog site|sars|nisar|"
                       r"discarded equipment|martian map|trend detective", re.I)


def fetch(name):
    path = os.path.join(RD, name.replace("/", "__") + ".md")
    if os.path.exists(path):
        return open(path, encoding="utf-8", errors="ignore").read()
    try:
        out = subprocess.run(["gh", "api", f"repos/{name}/readme", "-H", "Accept: application/vnd.github.raw"],
                             capture_output=True, timeout=60)
        text = out.stdout.decode("utf-8", errors="ignore") if out.returncode == 0 else ""
    except Exception:
        text = ""
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
    return text


def main():
    rows = []
    with open(os.path.join(HERE, "raw.tsv"), encoding="utf-8", errors="ignore") as f:
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) >= 5 and "/" in p[0]:
                rows.append(p)
    names = sorted({r[0] for r in rows})
    desc = {r[0]: r[3] for r in rows}
    stars = {r[0]: r[2] for r in rows}
    with ThreadPoolExecutor(8) as ex:
        texts = dict(zip(names, ex.map(fetch, names)))

    coded = []
    for n in names:
        t = (desc.get(n, "") + "\n" + texts[n])
        head = t[:3000]
        if not (RELEVANT.search(t) and HEALTH.search(t)) or OFF_TOPIC.search(head):
            continue
        rec = {"repo": n, "stars": stars.get(n, "0"), "readme_chars": len(texts[n])}
        low = t.lower()
        for k, pat in FEATURES.items():
            rec[k] = int(bool(re.search(pat, low)))
        coded.append(rec)

    keys = ["repo", "stars", "readme_chars"] + list(FEATURES)
    with open(os.path.join(HERE, "coded.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, keys)
        w.writeheader()
        w.writerows(coded)

    n = len(coded)
    substantive = [c for c in coded if c["readme_chars"] >= 1500]
    lines = [f"candidates: {len(names)}  relevant: {n}  substantive README (>=1500 chars): {len(substantive)}", ""]
    lines.append(f"{'feature':26s} {'all':>9s} {'substantive':>13s}")
    for k in FEATURES:
        a = sum(c[k] for c in coded)
        s = sum(c[k] for c in substantive)
        lines.append(f"{k:26s} {a:4d} {100*a/max(1,n):3.0f}%   {s:4d} {100*s/max(1,len(substantive)):3.0f}%")
    out = "\n".join(lines)
    open(os.path.join(HERE, "summary.txt"), "w", encoding="utf-8").write(out)
    print(out)
    assert n > 0 and all(c["repo"] for c in coded)


if __name__ == "__main__":
    sys.exit(main())
