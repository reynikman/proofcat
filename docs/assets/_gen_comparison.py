#!/usr/bin/env python3
"""Deterministic ProofCat comparison matrix -> SVG (light + dark).

Checkmarks and crosses are VECTOR PATHS, never font glyphs, so they can never
render as tofu boxes (the bug in the previous imagegen/raster version). Data is
transcribed from docs/COMPARISON.md; keep the two in sync by hand.
"""
from pathlib import Path
from xml.sax.saxutils import escape

COLS = ["Capability", "ProofCat", "OffShoot (Hedge)", "Silverstack / OM", "ShotPut Pro"]
# column left-x anchors
CX = [60, 620, 1010, 1410, 1810]
W = 2200

# each data cell: (kind, text)  kind in check|cross|neu|dim|dash
ROWS = [
    (["Same physical disk", "rejected"],
     [("check", "core rule"), ("dim", "not documented"), ("dim", "not documented"), ("dim", "not documented")]),
    (["Fail-closed on", "destination mismatch"],
     [("check", "withholds\nSAFE_TO_FORMAT"), ("cross", "continues with\nwarnings [1]"), ("dim", "not documented"), ("dim", "not documented")]),
    (["Independent", "destination read-back"],
     [("check", "always in ArchiveMax"), ("neu", "optional Source &\nDestination [1]"), ("neu", "optional setting [4]"), ("neu", "optional selectable [5]")]),
    (["Default mode verifies…"],
     [("neu", "full hash source +\ndestination"), ("neu", "file sizes only [1]"), ("neu", "checksum during copy;\ntoggleable [4]"), ("neu", "selectable [5]")]),
    (["BLAKE3"],
     [("check", ""), ("dim", "not documented [1]"), ("dim", "not documented [3]"), ("dim", "not documented [5]")]),
    (["Two hashes in one pass"],
     [("check", "XXH64 + BLAKE3"), ("dim", "not documented"), ("dim", "not documented"), ("dim", "not documented")]),
    (["xxHash"],
     [("check", "XXH64"), ("check", "XXH64BE [1]"), ("check", "[3][4]"), ("check", "[5]")]),
    (["MD5 / SHA-1 / C4"],
     [("dash", "—"), ("check", "legacy, optional [1]"), ("check", "MD5, SHA1 [3]"), ("check", "MD5, SHA [5]")]),
    (["ASC MHL manifest"],
     [("check", "per destination"), ("check", "verification /\ncreation mode [1][2]"), ("check", "[3]"), ("dim", "not confirmed")]),
    (["Multiple destinations"],
     [("check", ""), ("check", ""), ("check", "[3]"), ("check", "")]),
    (["Resume after crash /", "disconnect"],
     [("check", ""), ("check", "stop/resume [2]"), ("dim", "not confirmed"), ("dim", "not confirmed")]),
    (["Automatic re-copy", "after MHL issue"],
     [("check", "targeted repair"), ("check", "retry from\nsource/other dest [1]"), ("dim", "not documented"), ("dim", "not documented")]),
    (["Machine-readable", "verdict"],
     [("check", "SAFE_TO_FORMAT"), ("cross", "no equivalent\ndocumented"), ("cross", "no equivalent\ndocumented"), ("cross", "no equivalent\ndocumented")]),
]

THEMES = {
    "light": dict(bg="#F4EDE0", ink="#101010", dim="#8A857B", line="#DAD2C2",
                  hdr_bg="#101010", hdr_fg="#F4EDE0", green="#1A8A52", red="#C0392B", zebra="#EFE7D8"),
    "dark":  dict(bg="#101010", ink="#F4EDE0", dim="#9A948A", line="#2A2A2A",
                  hdr_bg="#F4EDE0", hdr_fg="#101010", green="#35C97E", red="#FF6B5E", zebra="#181818"),
}

FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"
LINEH = 34


def check_glyph(x, y, color):
    return (f'<path d="M{x+3},{y-1} L{x+10},{y+6} L{x+23},{y-9}" '
            f'fill="none" stroke="{color}" stroke-width="4.5" '
            f'stroke-linecap="round" stroke-linejoin="round"/>')


def cross_glyph(x, y, color):
    return (f'<path d="M{x+4},{y-8} L{x+20},{y+8} M{x+20},{y-8} L{x+4},{y+8}" '
            f'fill="none" stroke="{color}" stroke-width="4.5" stroke-linecap="round"/>')


def text_lines(x, y, lines, color, weight="400", size=25, family=FONT):
    out = []
    for i, ln in enumerate(lines):
        out.append(f'<text x="{x}" y="{y + i*LINEH}" font-family="{family}" '
                   f'font-size="{size}" font-weight="{weight}" fill="{color}">{escape(ln)}</text>')
    return "".join(out)


def build(theme_name):
    t = THEMES[theme_name]
    parts = []
    # header band + title
    parts.append(f'<text x="60" y="92" font-family="{FONT}" font-size="46" font-weight="700" fill="{t["ink"]}">ProofCat vs. the incumbents</text>')
    parts.append(f'<text x="60" y="138" font-family="{FONT}" font-size="24" fill="{t["dim"]}">Only documented claims. “not documented” means exactly that.</text>')

    hdr_y = 185
    parts.append(f'<rect x="40" y="{hdr_y}" width="{W-80}" height="58" fill="{t["hdr_bg"]}" rx="6"/>')
    for i, c in enumerate(COLS):
        parts.append(f'<text x="{CX[i]}" y="{hdr_y+38}" font-family="{FONT}" font-size="24" font-weight="600" fill="{t["hdr_fg"]}">{escape(c)}</text>')

    y = hdr_y + 58
    ri = 0
    for cap_lines, cells in ROWS:
        nlines = max([len(cap_lines)] + [c[1].count("\n") + 1 for c in cells])
        rh = nlines * LINEH + 22
        if ri % 2 == 1:
            parts.append(f'<rect x="40" y="{y}" width="{W-80}" height="{rh}" fill="{t["zebra"]}"/>')
        parts.append(f'<line x1="40" y1="{y}" x2="{W-40}" y2="{y}" stroke="{t["line"]}" stroke-width="1"/>')
        ty = y + 40
        # capability
        parts.append(text_lines(CX[0], ty, cap_lines, t["ink"], weight="600"))
        # data cells
        for ci, (kind, txt) in enumerate(cells):
            x = CX[ci + 1]
            lines = txt.split("\n") if txt else []
            if kind == "check":
                parts.append(check_glyph(x, ty - 8, t["green"]))
                parts.append(text_lines(x + 34, ty, lines, t["green"]))
            elif kind == "cross":
                parts.append(cross_glyph(x, ty - 8, t["red"]))
                parts.append(text_lines(x + 34, ty, lines, t["red"]))
            elif kind == "dash":
                parts.append(text_lines(x, ty, ["—"], t["dim"]))
            elif kind == "dim":
                parts.append(text_lines(x, ty, lines, t["dim"]))
            else:  # neu
                parts.append(text_lines(x, ty, lines, t["ink"]))
        y += rh
        ri += 1

    parts.append(f'<line x1="40" y1="{y}" x2="{W-40}" y2="{y}" stroke="{t["line"]}" stroke-width="1"/>')
    y += 48
    parts.append(f'<text x="60" y="{y}" font-family="{FONT}" font-size="20" fill="{t["dim"]}">Sources checked 13 July 2026 — every claim linked in docs/COMPARISON.md</text>')
    y += 42
    parts.append(f'<text x="60" y="{y}" font-family="{FONT}" font-size="22" fill="{t["ink"]}">Where incumbents are ahead: years in production · RAW colour · ecosystem · commercial support.</text>')
    y += 40

    H = y
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
           f'font-family="{FONT}"><rect width="{W}" height="{H}" fill="{t["bg"]}"/>' + "".join(parts) + "</svg>")
    return svg


here = Path(__file__).parent
for name in ("light", "dark"):
    (here / f"comparison-{name}.svg").write_text(build(name), encoding="utf-8")
    print(f"wrote comparison-{name}.svg")
