---
name: ANCHOR
description: A personal baseline drawn as terrain, on a night survey sheet with a 2700 K evening rendering.
colors:
  pine-ink-ground: "#0f221c"
  pine-surface: "#16302a"
  pine-rule: "#2c4a40"
  vellum-text: "#eee8d8"
  sage-muted: "#b9c2ad"
  contour-ochre: "#a98a5e"
  corridor-moss: "#5f8a52"
  cream-trace: "#f4efe2"
  lichen-accent: "#cfe3b4"
  on-accent: "#0f221c"
  advisory-blue: "#8cc4f0"
  caution-yellow: "#f0d84a"
  warning-red: "#ff8f80"
  on-signal: "#0f221c"
  evening-ground: "#0d0904"
  evening-surface: "#1a1209"
  evening-rule: "#3a2b14"
  evening-text: "#ffb858"
  evening-muted: "#d39548"
  evening-contour: "#8a6430"
  evening-corridor: "#6f6a1f"
  evening-trace: "#ffc458"
  evening-accent: "#ffb858"
  evening-on-accent: "#0d0904"
  evening-advisory: "#ffb858"
  evening-caution: "#f0c23a"
  evening-warning: "#ff6a3d"
  evening-on-signal: "#0d0904"
typography:
  headline:
    fontFamily: "Atkinson Hyperlegible, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "34px"
    fontWeight: 650
    lineHeight: 1.15
    letterSpacing: "-0.01em"
    fontFeature: "tnum, zero"
  title:
    fontFamily: "Atkinson Hyperlegible, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "21px"
    fontWeight: 650
    lineHeight: 1.25
    fontFeature: "tnum, zero"
  body:
    fontFamily: "Atkinson Hyperlegible, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "19px"
    fontWeight: 400
    lineHeight: 1.45
    fontFeature: "tnum, zero"
  body-small:
    fontFamily: "Atkinson Hyperlegible, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.45
    fontFeature: "tnum, zero"
  label:
    fontFamily: "Atkinson Hyperlegible, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 600
    lineHeight: 1.45
    letterSpacing: "0.08em"
    fontFeature: "tnum, zero"
rounded:
  fill: "4px"
  sm: "6px"
  md: "10px"
  pill: "999px"
spacing:
  xs: "8px"
  sm: "12px"
  md: "16px"
  lg: "24px"
  xl: "32px"
  xxl: "40px"
components:
  button-primary:
    backgroundColor: "{colors.lichen-accent}"
    textColor: "{colors.on-accent}"
    typography: "{typography.title}"
    rounded: "{rounded.md}"
    padding: "0 28px"
    height: "104px"
  button-secondary:
    backgroundColor: "{colors.pine-surface}"
    textColor: "{colors.vellum-text}"
    rounded: "{rounded.md}"
    padding: "0 28px"
    height: "80px"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.sage-muted}"
    rounded: "{rounded.md}"
    padding: "0 28px"
    height: "80px"
  button-disabled:
    backgroundColor: "{colors.pine-ink-ground}"
    textColor: "{colors.sage-muted}"
    rounded: "{rounded.md}"
    padding: "0 28px"
    height: "80px"
  segmented-option:
    backgroundColor: "transparent"
    textColor: "{colors.sage-muted}"
    padding: "0 18px"
    height: "80px"
    width: "80px"
  segmented-option-selected:
    backgroundColor: "{colors.lichen-accent}"
    textColor: "{colors.on-accent}"
  switch:
    backgroundColor: "{colors.pine-ink-ground}"
    textColor: "{colors.sage-muted}"
    rounded: "{rounded.md}"
    padding: "0 28px"
    height: "80px"
    width: "200px"
  switch-on:
    backgroundColor: "{colors.pine-surface}"
    textColor: "{colors.vellum-text}"
  bipolar-slider-track:
    backgroundColor: "{colors.pine-surface}"
    rounded: "{rounded.md}"
    height: "104px"
  message-card:
    backgroundColor: "{colors.pine-surface}"
    textColor: "{colors.vellum-text}"
    rounded: "{rounded.md}"
    padding: "20px 24px"
  chip:
    backgroundColor: "transparent"
    textColor: "{colors.sage-muted}"
    typography: "{typography.body-small}"
    rounded: "{rounded.pill}"
    padding: "6px 12px"
  rail-item:
    backgroundColor: "transparent"
    textColor: "{colors.sage-muted}"
    height: "104px"
  rail-item-active:
    backgroundColor: "{colors.pine-surface}"
    textColor: "{colors.vellum-text}"
  survey-sheet:
    backgroundColor: "{colors.pine-ink-ground}"
    rounded: "{rounded.md}"
    padding: "16px 12px 12px"
---

# Design System: ANCHOR

## Overview

**Creative North Star: "The Night Survey Sheet"**

ANCHOR is a topographic survey sheet read at night. The ground is pine ink. The person's own normal is drawn on it as terrain: a moss corridor for the usual range, an ochre dashed benchmark for the pre-flight level, a lichen hinge where a consented new normal begins, and a cream trace for the person's own values. Everything else on the sheet, from the rail and top line to the controls, is ruled, flat and quiet so that the terrain and the reserved signal colours stay legible.

The system is an Operate shell for a tablet held at arm's length in a moving, dim cabin. That sets its density: few elements per screen, 19 px body type, 80 px minimum touch targets and 104 px primary ones. Labels are set in uppercase map-collar marginalia, figures are tabular with slashed zeros, and meaning never depends on colour alone. In the evening the whole sheet re-renders in a 2700 K amber monochrome, and shape and label take over everything that colour carried in the day.

The system rejects rings, gauges, composite scores and red/green status cards. Signal colours belong to NASA-STD-3001 meanings and nothing else.

**Key Characteristics:**
- Flat, ruled panels on a single dark ground; depth comes from one tonal step and 1 px rules.
- One accent (lichen) for selection and the primary action; signal colours reserved for their meaning.
- Large, calm controls: 80 px minimum, 104 px primary, and drag-anywhere tracks.
- Dashed strokes mark reference, not-yet or not-live states throughout.
- Two complete token sets, day and evening, swapped at runtime on the same CSS custom properties.

## Colors

A dark, low-chroma green-black sheet with warm paper-toned ink, one pale lichen accent and three reserved signal hues. The evening set replaces every value with amber. Tokens live in `app/src/tokens.ts` and are written to `:root` as `--ground`, `--surface`, `--line`, `--text`, `--muted`, `--contour`, `--corridor`, `--trace`, `--accent`, `--on-accent`, `--advisory`, `--caution`, `--warning` and `--on-signal`. Components always reference the custom property, never a literal hex, so the evening swap reaches them.

### Primary
- **Lichen Accent** (`lichen-accent`; evening `evening-accent`): primary action fill, the selected segment, the active-rail underline, the focus ring, the hinge line and pin, the secondary-button border, text selection, caret and native accent colour. Text on it uses **On Accent**.

### Secondary
- **Corridor Moss** (`corridor-moss`; evening `evening-corridor`): the personal range only. It fills the corridor band on Course (at 0.55 opacity), the range in Today's range bars and the slider fill. It is never used for text or controls.
- **Cream Trace** (`cream-trace`; evening `evening-trace`): the person's own values. It draws the trace line and dots, tonight's mark in a range bar and the slider thumb.

### Tertiary
- **Contour Ochre** (`contour-ochre`; evening `evening-contour`): grid contours (at 0.55 opacity) and the dashed pre-flight benchmark, both on the chart and as the range-bar anchor. It draws lines only.

### Signal (reserved)
- **Warning Red** (`warning-red`; evening `evening-warning`): safety lines and the warning register (2 px card border, register word, diamond glyph, limit line).
- **Caution Yellow** (`caution-yellow`; evening `evening-caution`): the caution register and episode marks (1 px card border, triangle glyph, episode box and number).
- **Advisory Blue** (`advisory-blue`; day only): the outlined circle glyph and the register word for advisories. In the evening, `evening-advisory` equals the amber text colour.
- **On Signal** (`on-signal`): text on a caution or warning fill.

### Neutral
- **Pine Ink Ground** (`pine-ink-ground`; evening `evening-ground`): page background, chart sheets, the off state of a switch, the disabled-control fill.
- **Pine Surface** (`pine-surface`; evening `evening-surface`): one step up for message cards, slider tracks, the replay footer, the active rail item and the on state of a switch.
- **Pine Rule** (`pine-rule`; evening `evening-rule`): all 1 px rules and 2 px control borders, the slider track rail, the scrubber track and scrollbars.
- **Vellum Text** (`vellum-text`; evening `evening-text`): primary text.
- **Sage Muted** (`sage-muted`; evening `evening-muted`): secondary text, marginalia, axis labels, unselected controls and disabled text.

### Named Rules
**The Six-to-One Rule.** Every text and background pair is at least 6:1 in both day and evening modes, including disabled text. `app/test/tokens.test.ts` enforces this; a new text pairing must pass it before it ships.

**The 2700 K Rule.** No evening token may be bluer than a 2700 K source (blue channel at most 0.35 of the maximum channel). The evening set is amber monochrome; advisories rely on shape and label.

**The Reserved Signal Rule.** Warning red, caution yellow and advisory blue appear only for their NASA-STD-3001 meaning. They are never decoration, selection or brand.

**The Ochre-Is-Line Rule.** Contour ochre draws grid and benchmark lines only. It is never used as text or as a fill, and the token test fails if it is used as a text colour.

## Typography

**Display Font:** none. The type system has a single family.
**Body Font:** Atkinson Hyperlegible, self-hosted in Regular and Bold (fallback: Segoe UI Variable Text, Segoe UI, system-ui, SF Pro Text, Roboto, Helvetica Neue, Arial, sans-serif)

**Character:** A single legibility-first sans, chosen to tell apart l, 1, I and O, 0. It renders a slashed zero. Hierarchy comes from size, weight and uppercase tracked marginalia, never from a second face.

### Hierarchy
- **Headline** (650, 34px, 1.15, -0.01em, balanced wrap): one per view, the view title (h1).
- **Title** (650, 21px, 1.25): message-card headings, slider labels, Support question text, the primary-button label and top-line values.
- **Body** (400, 19px, 1.45): running text. The intro paragraph under a heading is capped at 62ch and set in muted colour.
- **Body Small** (400, 16px, 1.45): fine print (70ch cap), legends, table cells, axis labels, chips and footer text. 16 px is the floor; nothing is set smaller.
- **Label** (600, 16px, 0.08em, uppercase, muted): map-collar marginalia. It names top-line cells, section headings (h2) and inline lead-ins such as "Where it goes".

The wordmark is a single instance (750, 20px, 0.18em tracking; 17px and 0.1em below 900px).

### Named Rules
**The Tabular Figures Rule.** `tabular-nums slashed-zero` is set on `:root`, so every figure has the same width (ten repeats of all ten digits measure 120.0 px at 19 px) and every zero is slashed. Do not switch it off for any number.

**The Map-Collar Rule.** Uppercase tracked text is used only for marginalia: a cell or section label in the label style. It never sits stacked above an h1 as a kicker, and a message card's register word stays inline on its heading line.

## Layout

The shell is a CSS grid filling 100dvh. It has a full-width top line (minimum 84 px, cells divided by 1 px rules), a left rail (136 px) with 104 px items, a scrolling main column (padding 32 / 40 / 48 px, content capped at 1120 px, a 24 px gap between blocks) and, in replay, a full-width footer on the surface tone. Lists and settings are separated by 1 px bottom rules with 12–20 px vertical padding rather than boxed. Spacing steps are 8, 12, 16, 24, 32 and 40 px. Text measures are capped at 62–78ch.

Breakpoints:
- **900 px (portrait tablet):** the rail narrows to 104 px, the main padding drops to 24 / 20 / 40 px, and the top line wraps into two full rows, with the mode controls on their own ruled row. The footer becomes two columns, and range bars stack.
- **600 px (phone, not a target device):** a single column. The rail moves to the top as a horizontal row of 80 px items, the page scrolls as a whole, and the replay footer is sticky at the bottom edge with its long note dropped.

Charts size the SVG width to the sheet, so one SVG unit is one CSS px. Stroke widths and chart type sizes (16 px axes, 17 px hinge label) are therefore real pixels.

## Elevation & Depth

The system is flat. There are no drop shadows. Depth is a single tonal step from Pine Ink Ground to Pine Surface plus 1 px Pine Rule lines. The only box-shadow is structural: a 4 px inset accent underline marks the active rail item. Focus is a 3 px accent outline with a 3 px offset. Nothing animates. The stylesheet declares no transitions, and reduced-motion disables any that a browser would add.

### Named Rules
**The Flat Sheet Rule.** Surfaces never lift. A panel is distinguished by its tone, its rule or its border weight, never by a shadow.

## Shapes

Corners are gently rounded at a single radius (10px) for buttons, segmented controls, cards, slider tracks, sheets and the read-only note. Smaller radii are used only inside instruments: 6px for range-bar tracks and slider thumbs, 4px for fills. Chips are the only pill shape. Borders come in three weights with meaning: a 1 px rule for structure, a 2 px border for an interactive control's edge, and a 2 px signal border for a warning card.

Register glyphs are geometric and filled at 28px: a **diamond** for warning, a **triangle** for caution, an outlined **circle** for advisory and a horizontal **rule** for an update. Link and mode badges use 12px marks drawn in CSS: a solid dot for link up, a ring for loss of signal and a dashed square for synthetic replay.

### Named Rules
**The Dashed Edge Rule.** A dashed stroke means reference, not yet, or not live: the pre-flight benchmark, an unrated slider, a disabled control, the read-only note, the synthetic badge, loss of signal on the mission-day strip and an undone hinge. A solid stroke means the person's live state.

**The Shape-Carries-It Rule.** A register is carried by shape and word together, and colour only repeats it. This must survive the evening rendering, where advisory blue becomes amber.

## Components

### Buttons
Large, calm and unmistakable.
- **Shape:** gently rounded (10px).
- **Primary:** Lichen Accent fill and On Accent text, 650 weight and 21px, 104px minimum height and 28px horizontal padding. Use one per task, for example "Save check-in".
- **Secondary:** Pine Surface fill, a 2px Lichen Accent border and Vellum Text, with an 80px minimum height.
- **Ghost:** no fill, a 2px Pine Rule border and Sage Muted text, with an 80px minimum height.
- **Disabled:** Pine Ink Ground fill, Sage Muted text and a 2px **dashed** Pine Rule border, with a not-allowed cursor. The text stays at least 6:1. Opacity is never used.
- **Focus:** a 3px accent outline at a 3px offset. No hover styling is defined, because the target device is touch.

### Segmented Control
- **Style:** an inline row inside a 2px Pine Rule border at 10px radius. Options are 80 × 80px minimum, divided by 1px rules, in Sage Muted at 600 weight.
- **State:** the selected option fills with Lichen Accent and On Accent text (`role="radio"`, `aria-checked`). There is a full-width variant, a 160px-wide yes/no variant and a compact 64px top-line variant.

### Switch
- **Off:** Pine Ink Ground and a 2px Pine Rule border, with muted text.
- **On:** Pine Surface, an accent border, Vellum Text and a 12px accent dot before the label. The minimum width is 200px.

### Bipolar Slider (signature)
- A full-width track 104px tall (Pine Surface, a 2px Pine Rule border, 10px radius) that takes a press anywhere. The value follows the finger and commits on release.
- A 2px mid rule, a Corridor Moss fill from the middle and a 16px Cream Trace thumb. Words sit at both ends and in the middle in Body Small.
- **Unrated:** the border turns dashed and the fill and thumb are hidden. It shows "not rated", never a default of 50.

### Message Card
- **Corner Style:** 10px. **Background:** Pine Surface. **Border:** 1px Pine Rule by default, 1px Caution Yellow for caution and 2px Warning Red for warning. **Padding:** 20px 24px, capped at 78ch.
- The header row holds the 28px register glyph, then an h3 with the register word inline ("Safety line:", "Worth a look:"). A step line follows: "What you can do:" or "Who acts:" in Vellum Text, with the action in muted text.

### Chips
- **Style:** pill (999px), a 1px Pine Rule border, Sage Muted text in Body Small, 6px 12px padding.
- **State:** an item that is waiting or undeliverable switches its border and text to Vellum Text.

### Navigation
- **Rail:** a stack of 104px items (80px when horizontal on a phone). Each item has a 34px outline icon (1.8 stroke, round caps) above a 600-weight label, separated by 1px rules. Items are muted at rest. The current page (`aria-current="page"`) takes Pine Surface, Vellum Text and a 4px inset accent underline.
- **Top line:** ruled cells, each a marginalia label over a 21px/650 value: mission day, ground link and mode.

### Survey Sheet and Range Bar (signature)
- **Sheet:** a figure on Pine Ink Ground with a 1px rule and 10px radius. It contains the corridor chart (ochre contours, a moss corridor, a dashed ochre benchmark, an accent hinge with a pin and plate, a cream trace, a dotted red limit and caution bars for episodes, each labelled once beside its triangle) and the mission-day strip on the same time axis. A data table in a `details` element gives the same values in text.
- **Range bar:** the chart in miniature. A 40px track holds the moss range, a 3px dashed ochre pre-flight anchor and an 8px cream mark for tonight.

### Read-only state
- The replay bar says "read-only" once, and disabled controls keep their dashed border. There is no per-view read-only banner (removed in the product audit below).

## Do's and Don'ts

### Do:
- **Do** reference colours through the runtime custom properties (`var(--accent)`, `var(--line)`) so the evening set reaches every component.
- **Do** keep every text pair at 6:1 or better in both modes, and run `app/test/tokens.test.ts` after any token change.
- **Do** size touch targets at 80px minimum and 104px for the primary control on a screen.
- **Do** carry every register with its glyph shape (diamond, triangle, circle, rule) and its word, and let colour only repeat them.
- **Do** mark not-yet, reference and not-live states with a dashed stroke, and live state with a solid one.
- **Do** keep figures tabular with slashed zeros, and keep text at 16px or larger.
- **Do** size chart SVGs to the sheet width so one unit is one CSS px.

### Don't:
- **Don't** use warning red, caution yellow or advisory blue for anything but their NASA-STD-3001 meaning, and don't use blue at all in the evening set.
- **Don't** use contour ochre for text or fills.
- **Don't** fade disabled controls with opacity. Use muted text on the ground with a dashed border.
- **Don't** add drop shadows or lift surfaces. Depth is one tonal step and a rule.
- **Don't** introduce rings, gauges, composite scores or red/green status cards.
- **Don't** stack an uppercase label above a heading as a kicker. Marginalia labels cells and sections only.

## Revision, 14 Nov 2026: iPadOS conventions (Apple HIG pass)

This revision supersedes the shell and type sections above wherever they conflict. Colour tokens, the corridor/benchmark/hinge chart and every NASA-STD-3001 App. F rule are unchanged and still tested.

- **Navigation.**
  - **Regular width (≥ 1000 px):** a 300 px sidebar with icon-and-label rows, each 80 px tall; the selected row is filled with the accent colour. The sidebar also holds the Lighting and Data segmented controls. (HIG layout.md › Size classes: a tab bar becomes a sidebar when there is room.)
  - **Compact width:** the same four destinations in a bottom tab bar, with Lighting and Data under a "Display and data" disclosure in the status bar.
- **Status bar.** A sticky bar at the top of the content shows the mission day, the ground link and the synthetic-replay state as pills. It uses a translucent material over the scrolling content, and switches to solid when reduced transparency is requested.
- **Type scale.** HIG Dynamic Type proportions, scaled for arm's length and floored at 16 px:
  - Large Title 40 / 700;
  - Title 2 26 / 700;
  - Title 3 22 / 650;
  - Body 19;
  - Subhead 17;
  - Footnote 16.
- **Layout.**
  - The content column has a maximum width of 1240 px and is centred in the space beside the sidebar.
  - Views split into two columns when the content area is at least 1000 px wide. This is a container query: the decision follows available space, not the device.
  - Charts are drawn at the measured column width, so one SVG unit equals one CSS pixel.
- **Components.**
  - **Cards:** one surface colour, a 20 px radius and a 1 px rule.
  - **Segmented controls:** a track holding selected pills, 80 px tall.
  - **Pop-up button (measure picker on Course):** nine measures exceed HIG's "about five to seven segments" (segmented-controls.md), so Course uses a native pop-up button (pop-up-buttons.md), 80 px tall, grouped by source: evening check-in, wearable, morning measurements. The chart moves up by one row of controls, and the platform supplies the menu, keyboard and screen-reader behaviour.
  - **Switches:** a 66 × 40 px track whose knob position, fill and word all state the setting (R2); the whole control is the 80 px target.
  - **Sharing settings:** inset grouped lists.
  - **Chart cards:** open with a title and a one-sentence main message; the coverage and episode details sit below the chart (HIG charts.md).
- **Tonight's insight** is one sentence at the top of the range card ("Your mood tonight is inside your usual range…"), not a card of its own.

## Product audit, 14 Nov 2026

Every screen was captured in every state (iPad landscape and phone, day and evening, replay and live) and read against the requirements in docs/UI_RESEARCH.md, Apple's HIG and NASA-STD-3001. Each change below removes something or fixes a contradiction; nothing was added for decoration.

| Finding | Principle | Change |
|---|---|---|
| The evening check-in showed notes about the person's own data before they rated. | UI_RESEARCH R13: rate first, then see the data. | Until the check-in is saved, only safety notices show; a line says how many notes wait. |
| Three identical "stayed lower" cards on one day. | Calm by construction (I3) and DESIGN_PAPER F4 (few proactive messages). HIG managing-notifications offers the same idea as delivery "in a summary" instead of separate alerts. | Changes open at once form one note; its "Why this note?" lists each measure with its start day and threshold. |
| Every sustained-change note blamed "a heavy work block or shorter sleep", also for blood oxygen. | Explainability: a cause is stated only where the research supports it (HRP_MAPPING.md). | Cause hints per kind of measure; body measures get none and point to the flight surgeon if a change stays. |
| An untouched vitals stepper saved yesterday's value as today's. | Data integrity; HIG entering-data: prefill a reasonable default, then let people confirm. | The last reading is shown muted and labelled; tapping it keeps it; only set values are saved. |
| Nine weekly-summary rows in the sharing log read the same. | Disclosure contract: every share listed, but readable. | Identical consecutive shares collapse into one row that still names every day. |
| "Stays on this tablet" and "agreed before flight" repeated on every row. | HIG writing: say it once, where it applies. | One sentence per group; rows carry only what differs. |
| The screener questions were always open, although the text says "asked only when you choose". | HIG disclosure-controls: hide details until they are relevant; consistency between words and behaviour. | The two questions sit behind a disclosure the astronaut opens. |
| Course offered "new normal" and "undo" side by side, both disabled. | HIG buttons: one clear action. | Only the action that applies is shown. |
| A read-only banner on every view. | Simplicity. | Said once, in the replay bar. |
| Episode marks were empty boxes plus numbered labels elsewhere. | R2: shape and words together, no lookup. | A caution bar over the days, labelled once beside its triangle. |
| Phone: status bar in four rows and replay bar in three; 330 px left for content. | HIG layout: content first; controls at least 44 pt. | Two-row status bar, one-row replay bar; 520 px for content. Full labels stay as accessible names. |
| `verify.sh` passed with a TypeScript error. | Evidence must be able to fail. | One command per line, so `set -e` stops on any failure. |
