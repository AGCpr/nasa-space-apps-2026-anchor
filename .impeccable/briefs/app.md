# Surface brief: ANCHOR app (`app/index.html`)

Mode: **Operate**. Build path: code-led (Operate surfaces are always code-led). Decided in docs/UI_RESEARCH.md §3–5 (direction A chosen by the user on 14 Nov 2026) and ADR-0009.

## Direction contract

THESIS: The person's own normal, drawn as terrain. Each value is a moss corridor (the personal range, 80% predictive band), the pre-flight anchor is a dashed benchmark line, and a consented re-anchoring is a hinge where the benchmark moves. This replaces the category default of rings, gauges, composite scores and red/green status cards.

OWN-WORLD: A topographic survey sheet at night. The ground is pine ink, contour and benchmark lines are ochre, the corridor is moss, and the person's own values are drawn in a cream trace. The type is sans with uppercase map-collar marginalia and tabular, slashed-zero figures. NASA-STD-3001 App. F colours are reserved for their meaning only: warning red, caution yellow, advisory blue (day only). In the evening rendering the palette shifts to a 2700 K amber monochrome.

STORY: The astronaut rates first, then sees tonight placed in their own range. They can see where a sustained change was noticed, accept a new normal or undo it, and see everything that left the tablet before it left. A person can be reached at any time.

FIRST VIEWPORT: The Today screen. The top line shows mission day, ground link and a "Synthetic replay" badge. A left rail holds Today, Course, Sharing and Support. The main area holds the check-in: three full-width bipolar sliders, each 104 px tall, and a 104 px "Save check-in" button. After saving, it shows one insight and three range bars. In replay, the scrubber footer sits at the bottom.

FORM: Operate shell with a signature move: the corridor/benchmark/hinge chart on Course, repeated in miniature as Today's range bars, plus the mission-day strip on the shared time axis. seed: none. No concept roll was run: the direction was chosen by the user from four research-grounded directions in docs/UI_RESEARCH.md §4–5 (14 Nov 2026), before this skill was involved; the roll is not re-run. The user confirmed this on 14 Nov 2026, choosing the option worded: “Yes, A stands”, meaning that the research-based choice replaces the roll and no new direction is opened.

Signature interaction: a drag-anywhere slider that commits on release; "This is my new normal" draws a hinge.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
