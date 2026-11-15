#!/usr/bin/env python3
"""Generates the .dc.html artboards of the Potagerie canvas (static isometric diorama)."""
import json, os, datetime

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "project")
os.makedirs(ROOT, exist_ok=True)

# ---------- Palette ----------
C = dict(
    bg="#EEF2E6", card="#E3ECDD", ink="#1F3A2E", muted="#56705F", line="#D3DECD",
    white="#FFFFFF", accent="#F2A65A", accentInk="#5A3410", green="#2F7D5B",
    grassTop="#9CCB86", grassL="#7FB36E", grassR="#6A9F5E",
    soilL="#8A5E3C", soilR="#74492D",
    bedTop="#A0704A", bedFurrow="#8E5F3D", bedL="#7E5236", bedR="#6B432B",
    woodTop="#D9A86E", woodL="#C08A53", woodR="#A87442",
    crit="#C8443C", warn="#D9822B", good="#2F8A5F", frost="#8DBFE0",
)

TW, TH = 48, 24  # tuile 50 cm


class Iso:
    def __init__(self, ox, oy, k=1.0):
        self.ox, self.oy, self.k = ox, oy, k

    def p(self, gx, gy, z=0):
        k = self.k
        return (self.ox + (gx - gy) * TW / 2 * k, self.oy + (gx + gy) * TH / 2 * k - z * k)


def pts(*ps):
    return " ".join(f"{x:.1f},{y:.1f}" for x, y in ps)


def poly(ps, fill, extra=""):
    return f'<polygon points="{pts(*ps)}" fill="{fill}"{extra}></polygon>'


def box(iso, x, y, w, h, z0, z1, top, left, right, top_extra=""):
    P = iso.p
    s = []
    s.append(poly([P(x, y + h, z1), P(x + w, y + h, z1), P(x + w, y + h, z0), P(x, y + h, z0)], left))
    s.append(poly([P(x + w, y, z1), P(x + w, y + h, z1), P(x + w, y + h, z0), P(x + w, y, z0)], right))
    s.append(poly([P(x, y, z1), P(x + w, y, z1), P(x + w, y + h, z1), P(x, y + h, z1)], top, top_extra))
    return "".join(s)


# ---------- plantes low-poly ----------
def gem(cx, cy, r, h, light, dark, top=None):
    """Faceted block: left half light, right half dark, top facet."""
    L = [(cx, cy - h), (cx - r, cy - h * 0.55), (cx - r * 0.85, cy - h * 0.12), (cx, cy)]
    R = [(cx, cy - h), (cx + r, cy - h * 0.55), (cx + r * 0.85, cy - h * 0.12), (cx, cy)]
    s = poly(L, light) + poly(R, dark)
    if top:
        s += poly([(cx, cy - h), (cx - r * 0.55, cy - h * 0.72), (cx, cy - h * 0.5), (cx + r * 0.55, cy - h * 0.72)], top)
    return s


def cone(cx, cy, r, h, light, dark):
    return poly([(cx, cy - h), (cx - r, cy), (cx, cy + r * 0.32)], light) + poly([(cx, cy - h), (cx + r, cy), (cx, cy + r * 0.32)], dark)


def shadow(cx, cy, rx, k):
    return f'<ellipse cx="{cx:.1f}" cy="{cy + 1:.1f}" rx="{rx * k:.1f}" ry="{rx * 0.45 * k:.1f}" fill="#2A4A30" fill-opacity="0.18"></ellipse>'


def fruit(cx, cy, r, col, hi="#FFFFFF"):
    return (f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{r:.1f}" fill="{col}"></circle>'
            f'<circle cx="{cx - r * 0.35:.1f}" cy="{cy - r * 0.35:.1f}" r="{r * 0.32:.1f}" fill="{hi}" fill-opacity="0.55"></circle>')


def plant(kind, stage, cx, cy, k=1.0):
    """stage: 0 sowing, 1 young shoot, 2 adult, 3 harvest"""
    g = lambda v: v * k
    if stage == 0:
        return (poly([(cx, cy), (cx - g(5), cy - g(4)), (cx - g(1), cy - g(2))], "#7CC46A")
                + poly([(cx, cy), (cx + g(5), cy - g(5)), (cx + g(1), cy - g(2))], "#5FAA55")
                + f'<line x1="{cx:.1f}" y1="{cy:.1f}" x2="{cx:.1f}" y2="{cy - g(3):.1f}" stroke="#4C8F48" stroke-width="{g(1.2):.1f}"></line>')
    if stage == 1:
        light, dark = {"carotte": ("#A3D677", "#86C05E"), "poireau": ("#8BC7A0", "#6CAE86")}.get(kind, ("#8DCB6C", "#6DB257"))
        return shadow(cx, cy, 5, k) + gem(cx, cy, g(5.5), g(8), light, dark)
    s = ""
    if kind == "tomate":
        s += shadow(cx, cy, 11, k)
        s += f'<line x1="{cx + g(2):.1f}" y1="{cy:.1f}" x2="{cx + g(2):.1f}" y2="{cy - g(40):.1f}" stroke="#9C6B3E" stroke-width="{g(2):.1f}" stroke-linecap="round"></line>'
        s += gem(cx, cy - g(2), g(10), g(20), "#5FAE5B", "#478F4B")
        s += gem(cx - g(1), cy - g(14), g(8.5), g(18), "#6BBA62", "#4F9A50", "#86CB74")
        if stage >= 2:
            s += fruit(cx - g(5), cy - g(10), g(3.2), "#E5533D") + fruit(cx + g(5), cy - g(18), g(3), "#E86A45") + fruit(cx - g(2), cy - g(25), g(2.6), "#F08A4B")
    elif kind == "basilic":
        s += shadow(cx, cy, 7, k) + gem(cx, cy, g(7), g(12), "#74C66A", "#58AD56", "#94D884")
        s += f'<circle cx="{cx:.1f}" cy="{cy - g(13):.1f}" r="{g(1.6):.1f}" fill="#F6F1E6"></circle>'
    elif kind == "courge":
        s += shadow(cx, cy, 16, k)
        s += gem(cx - g(8), cy + g(1), g(10), g(9), "#5E9F52", "#4A8745")
        s += gem(cx + g(8), cy - g(1), g(10), g(10), "#68A95A", "#4F8C48", "#7DBA6A")
        if stage >= 3:
            s += gem(cx, cy + g(3), g(7), g(9), "#F0A048", "#D9822F", "#F6BC6E")
    elif kind == "carotte":
        s += shadow(cx, cy, 7, k)
        for dx, h in ((-4, 14), (0, 18), (4, 13)):
            s += cone(cx + g(dx), cy, g(2.6), g(h), "#8ACD62", "#6DB24D")
        if stage >= 2:
            s += poly([(cx - g(3), cy - g(1)), (cx + g(3), cy - g(1)), (cx, cy + g(3))], "#EF8A2E")
    elif kind == "oignon":
        s += shadow(cx, cy, 6, k)
        for dx, h in ((-3, 20), (0, 24), (3, 18)):
            s += f'<line x1="{cx:.1f}" y1="{cy - g(4):.1f}" x2="{cx + g(dx * 1.6):.1f}" y2="{cy - g(h):.1f}" stroke="#7DB56B" stroke-width="{g(2):.1f}" stroke-linecap="round"></line>'
        s += gem(cx, cy, g(5), g(7), "#E2AE6C", "#C98F4E", "#EEC690")
    elif kind == "haricot":
        s += shadow(cx, cy, 7, k)
        s += f'<line x1="{cx:.1f}" y1="{cy:.1f}" x2="{cx:.1f}" y2="{cy - g(42):.1f}" stroke="#A07448" stroke-width="{g(1.8):.1f}" stroke-linecap="round"></line>'
        s += cone(cx, cy - g(2), g(7), g(34), "#79C063", "#5BA54F")
    elif kind == "fraisier":
        s += shadow(cx, cy, 8, k) + gem(cx, cy, g(8.5), g(9), "#4E9A55", "#3D8247", "#64B067")
        if stage >= 3:
            s += fruit(cx - g(4), cy - g(2), g(2.4), "#E0384A") + fruit(cx + g(4), cy - g(4), g(2.2), "#E0384A")
    elif kind == "laitue":
        s += shadow(cx, cy, 9, k) + gem(cx, cy, g(9), g(11), "#B4DC7A", "#92C65E", "#CFEA9C")
    elif kind == "poireau":
        s += shadow(cx, cy, 5, k)
        s += poly([(cx - g(2), cy), (cx + g(2), cy), (cx + g(2), cy - g(12)), (cx - g(2), cy - g(12))], "#EEF3DA")
        s += cone(cx, cy - g(10), g(5), g(16), "#6FB487", "#559C70")
    return s


import sys as _sys
_sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from plants2 import plant  # noqa: E402  (replaces the simple version above)


def tree(iso, gx, gy):
    cx, cy = iso.p(gx, gy, 0)
    k = iso.k
    s = shadow(cx, cy, 26, k)
    s += f'<rect x="{cx - 3 * k:.1f}" y="{cy - 26 * k:.1f}" width="{6 * k:.1f}" height="{26 * k:.1f}" fill="#8A5A36"></rect>'
    s += cone(cx, cy - 22 * k, 26 * k, 50 * k, "#5E9F5A", "#4A8650")
    s += cone(cx, cy - 52 * k, 20 * k, 40 * k, "#6DAE63", "#56965A")
    s += cone(cx, cy - 80 * k, 13 * k, 30 * k, "#7DBE6E", "#62A35F")
    return s


def barrel(iso, gx, gy):
    cx, cy = iso.p(gx, gy, 0)
    k = iso.k
    r, h = 11 * k, 26 * k
    return (shadow(cx, cy, 14, k)
            + f'<rect x="{cx - r:.1f}" y="{cy - h:.1f}" width="{2 * r:.1f}" height="{h:.1f}" fill="#4F7FA6"></rect>'
            + f'<rect x="{cx:.1f}" y="{cy - h:.1f}" width="{r:.1f}" height="{h:.1f}" fill="#41698A"></rect>'
            + f'<ellipse cx="{cx:.1f}" cy="{cy:.1f}" rx="{r:.1f}" ry="{r * 0.5:.1f}" fill="#41698A"></ellipse>'
            + f'<ellipse cx="{cx:.1f}" cy="{cy - h:.1f}" rx="{r:.1f}" ry="{r * 0.5:.1f}" fill="#7FB3D6"></ellipse>')


def compost(iso, gx, gy):
    return (box(iso, gx, gy, 1.6, 1.6, 0, 14, "#7A5236", C["woodL"], C["woodR"])
            + box(iso, gx + 0.15, gy + 0.15, 1.3, 1.3, 14, 15, "#5E3F28", "#5E3F28", "#5E3F28"))


# ---------- garden data (example, 2027 season, on 20 June) ----------
NAMES = dict(A="Back of the garden", B="The tomatoes", C="The zucchinis", D="Roots", E="Shed side", F="Leek square", G="Salad square")

PLANCHES = [
    dict(id="A", x=1, y=1, w=8, h=2, kind="bed"),
    dict(id="D", x=11, y=1, w=8, h=2, kind="bed"),
    dict(id="B", x=1, y=4, w=8, h=2, kind="bed"),
    dict(id="E", x=11, y=4, w=8, h=2, kind="bed"),
    dict(id="C", x=1, y=7, w=8, h=2, kind="bed"),
    dict(id="F", x=11, y=7, w=4, h=4, kind="bac"),
    dict(id="G", x=16, y=7, w=3, h=4, kind="bac"),
]


def plants_for(pid):
    """list of (kind, stage, gx, gy) in absolute grid coordinates"""
    out = []
    P = {p["id"]: p for p in PLANCHES}[pid]
    x0, y0 = P["x"], P["y"]
    if pid == "B":
        for i in range(4):
            out.append(("tomate", 2, x0 + i + 0.5, y0 + 1.0))
        for i in range(4, 8):
            for j in (0.5, 1.5):
                out.append(("laitue", 2, x0 + i + 0.5, y0 + j))
    elif pid == "C":
        for i in (1, 3, 5, 7):
            out.append(("courgette", 3, x0 + i, y0 + 1.0))
    elif pid == "D":
        for i in range(8):
            for j in (0.5, 1.5):
                out.append(("carotte", 2, x0 + i + 0.5, y0 + j))
    elif pid == "E":
        for i in range(8):
            out.append(("haricot", 1, x0 + i + 0.5, y0 + 0.6))
            out.append(("haricot", 1, x0 + i + 0.5, y0 + 1.4))
    elif pid == "F":
        for i in range(4):
            for j in range(4):
                out.append(("poireau", 1, x0 + i + 0.5, y0 + j + 0.5))
    elif pid == "G":
        for i in range(3):
            for j in range(4):
                out.append(("laitue", 3, x0 + i + 0.5, y0 + j + 0.5))
    return out


def planche_svg(iso, p, selected=False, free=False):
    x, y, w, h = p["x"], p["y"], p["w"], p["h"]
    s = ""
    if p["kind"] == "bed":
        top_z = 7
        s += box(iso, x, y, w, h, 0, top_z, C["bedTop"], C["bedL"], C["bedR"])
        # sillons
        for j in range(1, int(h * 2)):
            a, b = iso.p(x + 0.15, y + j / 2, top_z), iso.p(x + w - 0.15, y + j / 2, top_z)
            s += f'<line x1="{a[0]:.1f}" y1="{a[1]:.1f}" x2="{b[0]:.1f}" y2="{b[1]:.1f}" stroke="{C["bedFurrow"]}" stroke-width="{1.4 * iso.k:.1f}" stroke-linecap="round"></line>'
    else:
        top_z = 12
        s += box(iso, x, y, w, h, 0, 14, C["woodTop"], C["woodL"], C["woodR"])
        s += box(iso, x + 0.18, y + 0.18, w - 0.36, h - 0.36, 13, 13.5, C["bedTop"], C["bedTop"], C["bedTop"])
    P = iso.p
    if selected:
        s += poly([P(x, y, top_z + 1), P(x + w, y, top_z + 1), P(x + w, y + h, top_z + 1), P(x, y + h, top_z + 1)], "none",
                  f' stroke="{C["accent"]}" stroke-width="{3 * iso.k:.1f}" stroke-linejoin="round"')
    if free:
        s += poly([P(x + 0.2, y + 0.2, top_z + 0.5), P(x + w - 0.2, y + 0.2, top_z + 0.5), P(x + w - 0.2, y + h - 0.2, top_z + 0.5), P(x + 0.2, y + h - 0.2, top_z + 0.5)], "#FFFFFF",
                  f' fill-opacity="0.12" stroke="#FFFFFF" stroke-width="{1.6 * iso.k:.1f}" stroke-dasharray="{6 * iso.k:.1f} {5 * iso.k:.1f}"')
    return s, top_z


def label_pin(iso, p, top_z, text, color=None, badge=None):
    # badge set on the left corner of the bed
    cx, cy = iso.p(p["x"] + 0.2, p["y"] + p["h"] - 0.2, top_z + 4)
    k = iso.k
    fs = 12 * k
    wpx = (len(text) * 6.6 + 20) * k
    lx, ly = cx - wpx + 6 * k, cy - 17 * k
    s = (f'<rect x="{lx:.1f}" y="{ly:.1f}" width="{wpx:.1f}" height="{22 * k:.1f}" rx="{11 * k:.1f}" fill="{color or C["white"]}"></rect>'
         f'<text x="{lx + wpx / 2:.1f}" y="{ly + 15 * k:.1f}" text-anchor="middle" font-family="Fredoka, sans-serif" font-weight="600" font-size="{fs:.1f}" fill="{C["ink"]}">{text}</text>')
    if badge:
        bx, by = lx + wpx - 2 * k, ly - 2 * k
        s += (f'<circle cx="{bx:.1f}" cy="{by:.1f}" r="{7.5 * k:.1f}" fill="{badge}" stroke="#FFFFFF" stroke-width="{2 * k:.1f}"></circle>'
              f'<text x="{bx:.1f}" y="{by + 3.4 * k:.1f}" text-anchor="middle" font-family="Figtree, sans-serif" font-weight="700" font-size="{10 * k:.1f}" fill="#FFFFFF">!</text>')
    return s


def free_sign(iso, p, top_z):
    cx, cy = iso.p(p["x"] + p["w"] / 2, p["y"] + p["h"] / 2, top_z)
    k = iso.k
    return (f'<line x1="{cx:.1f}" y1="{cy:.1f}" x2="{cx:.1f}" y2="{cy - 30 * k:.1f}" stroke="#8A5A36" stroke-width="{2.4 * k:.1f}" stroke-linecap="round"></line>'
            f'<rect x="{cx - 34 * k:.1f}" y="{cy - 52 * k:.1f}" width="{68 * k:.1f}" height="{24 * k:.1f}" rx="{8 * k:.1f}" fill="#FFFFFF"></rect>'
            f'<text x="{cx:.1f}" y="{cy - 36 * k:.1f}" text-anchor="middle" font-family="Figtree, sans-serif" font-weight="700" font-size="{11 * k:.1f}" fill="{C["green"]}">+ Free</text>')


def diorama(k=1.0, W=980, H=600, ox=None, oy=None, labels=True):
    ox = ox if ox is not None else 13 * TW / 2 * k + 60 * k
    oy = oy if oy is not None else 110 * k
    iso = Iso(ox, oy, k)
    GW, GH = 22, 13
    s = []
    # socle : terre puis herbe
    s.append(box(iso, 0, 0, GW, GH, -30, -8, C["grassTop"], C["soilL"], C["soilR"]))
    s.append(box(iso, 0, 0, GW, GH, -8, 0, C["grassTop"], C["grassL"], C["grassR"]))
    # small tufts and flowers
    import random
    rnd = random.Random(7)
    occupied = lambda gx, gy: any(p["x"] - 0.2 <= gx <= p["x"] + p["w"] + 0.2 and p["y"] - 0.2 <= gy <= p["y"] + p["h"] + 0.2 for p in PLANCHES)
    for _ in range(70):
        gx, gy = rnd.uniform(0.4, GW - 0.4), rnd.uniform(0.4, GH - 0.4)
        if occupied(gx, gy) or (19.6 < gx and gy < 3.4):
            continue
        cx, cy = iso.p(gx, gy, 0)
        if rnd.random() < 0.25:
            col = rnd.choice(["#F6E58A", "#F3B3C6", "#FFFFFF"])
            s.append(f'<circle cx="{cx:.1f}" cy="{cy - 2 * k:.1f}" r="{2 * k:.1f}" fill="{col}"></circle>')
        else:
            s.append(poly([(cx - 3 * k, cy), (cx - 1 * k, cy - 6 * k), (cx, cy)], "#86BC72") + poly([(cx, cy), (cx + 1.5 * k, cy - 7 * k), (cx + 3 * k, cy)], "#74AA63"))
    # beds by row (from back to front)
    rows = sorted(PLANCHES, key=lambda p: (p["y"], p["x"]))
    for p in rows:
        svg, top_z = planche_svg(iso, p, selected=(p["id"] == "B"), free=(p["id"] == "A"))
        s.append(svg)
        for kind, st, gx, gy in sorted(plants_for(p["id"]), key=lambda t: t[2] + t[3]):
            cx, cy = iso.p(gx, gy, top_z)
            s.append(plant(kind, st, cx, cy, k))
        if p["id"] == "A":
            s.append(free_sign(iso, p, top_z))
        if labels:
            badge = None
            s.append(label_pin(iso, p, top_z, NAMES[p["id"]] if labels != "short" else p["id"], C["accent"] if p["id"] == "B" else None, badge))
    body = "".join(s)
    return (f'<svg viewBox="0 0 {W} {H}" width="100%" style="display: block; max-width: {W}px; margin: 0 auto" '
            f'role="img" aria-label="Isometric view of the garden on 20 June 2027: seven beds with tomatoes, lettuces, zucchinis, carrots, beans and leeks">{body}</svg>')


# ---------- frise (Gantt) ----------
Y = 2027
DAYS = 365
MONTHS = ["Jan.", "Feb.", "Mar", "Apr.", "May", "Jun", "Jul.", "Aug", "Sept.", "Oct.", "Nov.", "Dec."]
MDAYS = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]


def doy(md):
    d = datetime.date(Y, int(md[:2]), int(md[3:]))
    return (d - datetime.date(Y, 1, 1)).days


def pct(md):
    return doy(md) / DAYS * 100


def mix(hex_, t):  # toward white
    h = hex_.lstrip("#")
    r, g, b = int(h[:2], 16), int(h[2:4], 16), int(h[4:], 16)
    r, g, b = [round(v + (255 - v) * t) for v in (r, g, b)]
    return f"#{r:02X}{g:02X}{b:02X}"


CROPCOL = dict(courgette="#3F8F4A", tomate="#E5533D", basilic="#58AD56", courge="#E89A3C", carotte="#EF8A2E", mache="#4F9A3A",
               poireau="#6FB487", laitue="#92C65E", haricot="#6DB24D", oignon="#C98F4E", fraisier="#E0384A", pois="#7FC94A")

# (planche, lane, nom, culture, abri, debut, recolte, fin, alerte)
SERIES = [  # dates computed per S03 with the durations of S02
    ("A", 0, "Lettuce", "laitue", None, "03-01", "05-18", "06-06", None),
    ("B", 0, "Tomato ½", "tomate", "03-10", "05-05", "07-19", "09-26", None),
    ("B", 1, "Lettuce ½", "laitue", "04-15", "05-13", "07-02", "07-21", None),
    ("C", 0, "Zucchini", "courgette", "04-10", "05-01", "06-20", "09-07", None),
    ("D", 0, "Carrot", "carotte", None, "03-15", "06-23", "08-01", None),
    ("D", 0, "Lettuce", "laitue", None, "08-15", "11-01", "11-20", None),
    ("E", 0, "Leek", "poireau", None, "01-01", "01-01", "02-04", None),
    ("E", 0, "Lettuce", "laitue", "02-28", "03-28", "05-17", "06-05", None),
    ("E", 0, "Green bean", "haricot", None, "06-10", "08-09", "09-07", None),
    ("F", 0, "Leek", "poireau", "03-10", "05-19", "09-16", "12-14", None),
    ("G", 0, "Lettuce (plants)", "laitue", None, "04-20", "06-09", "06-28", None),
    ("G", 0, "Lettuce", "laitue", "07-01", "07-29", "09-17", "10-06", None),
]
ROWS = [("A", "Back of the garden", "Bed · 4 × 1 m"), ("B", "The tomatoes", "Bed · 4 × 1 m"), ("C", "The zucchinis", "Bed · 4 × 1 m"),
        ("D", "Roots", "Bed · 4 × 1 m"), ("E", "Shed side", "Bed · 4 × 1 m"), ("F", "Leek square", "Bac · 2 × 2 m"), ("G", "Salad square", "Bac · 1.5 × 2 m")]
LANE_H = 34


def row_top(pid):
    y = 0
    for rid, _, _ in ROWS:
        if rid == pid:
            return y
        lanes = 1 + max([x[1] for x in SERIES if x[0] == rid] + [0])
        y += lanes * LANE_H + 10 + 1
    return y


def annotations():
    l, r = pct("08-01"), pct("08-21")
    out = (f'<div style="position: absolute; top: 0; bottom: 0; left: {l:.2f}%; width: {r - l:.2f}%; background: rgba(139,120,208,0.10); border-left: 1px dashed #8B78D0; border-right: 1px dashed #8B78D0"></div>'
           f'<div style="position: absolute; bottom: -30px; left: {l:.2f}%; width: {r - l + 6:.2f}%; padding: 3px 8px; border-radius: 8px; background: #ECE6FA; color: #4B3C7A; font-size: 11px; font-weight: 700; white-space: nowrap">Holidays · 1 → 21 Aug</div>')
    for pid, md, txt in (("B", "07-01", "Mulch the tomatoes"), ("C", "05-25", "Protect from slugs"), ("F", "04-10", "Weed")):
        x = pct(md)
        top = row_top(pid) + 3
        out += (f'<div style="position: absolute; top: {top}px; left: {x:.2f}%; transform: translateX(-7px); display: flex; align-items: flex-start; gap: 0; z-index: 2">'
                f'<span style="width: 14px; height: 14px; border-radius: 50% 50% 50% 0; background: #E9C02E; border: 2px solid #FFFFFF; box-sizing: border-box; transform: rotate(-45deg); margin-top: 2px"></span>'
                f'<span style="margin-left: 4px; padding: 2px 8px; border-radius: 8px; background: #FFF1BF; color: #5A470A; font-size: 11px; font-weight: 700; white-space: nowrap; box-shadow: 0 1px 3px rgba(90,71,10,0.18)">{txt}</span></div>')
    return out


def gantt(compact=False):
    cur = pct("06-20")
    cols = " ".join(f"{d}fr" for d in MDAYS)
    months = "".join(
        f'<div style="font-size: 11px; font-weight: 600; letter-spacing: 0.04em; text-transform: uppercase; color: {C["muted"]}; padding: 0 0 8px 6px; border-left: 1px solid {C["line"]}">{m}</div>'
        for m in MONTHS)
    labels, tracks = [], []
    for pid, name, dim in ROWS:
        lanes = 1 + max([s[1] for s in SERIES if s[0] == pid] + [0])
        hgt = lanes * LANE_H + 10
        sel = pid == "B"
        labels.append(
            f'<div style="height: {hgt}px; display: flex; flex-direction: column; justify-content: center; padding: 0 12px; '
            f'border-top: 1px solid {C["line"]}; {"background: #FDF1E4; box-shadow: inset 3px 0 0 " + C["accent"] + ";" if sel else ""}">'
            f'<span style="font-weight: 600; font-size: 14px; color: {C["ink"]}">{name}</span>'
            f'<span style="font-size: 12px; color: {C["muted"]}">{dim}</span></div>')
        bars = ""
        for (p, lane, nm, crop, abri, deb, rec, fin, alert) in SERIES:
            if p != pid:
                continue
            a = abri or deb
            left, right = pct(a), (doy(fin) + 1) / DAYS * 100
            dA = doy(deb) - doy(a)
            dG = doy(rec) - doy(deb)
            dH = doy(fin) - doy(rec) + 1
            col = CROPCOL[crop]
            segs = ""
            if dA > 0:
                segs += f'<div style="flex: {dA} 1 0; background: repeating-linear-gradient(135deg, {mix(col, 0.55)} 0 4px, {mix(col, 0.85)} 4px 8px)"></div>'
            if dG > 0:
                segs += f'<div style="flex: {dG} 1 0; background: {mix(col, 0.45)}"></div>'
            segs += f'<div style="flex: {dH} 1 0; background: {col}"></div>'
            badge = "" if True else (f'<span style="margin-left: 6px; display: inline-flex; align-items: center; gap: 4px; padding: 1px 7px; border-radius: 99px; background: #FFFFFF; color: {C["warn"]}; font-size: 11px; font-weight: 700">'
                     f'<svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><circle cx="5" cy="5" r="5" fill="{C["warn"]}"></circle><rect x="4.25" y="2" width="1.5" height="3.6" rx="0.7" fill="#fff"></rect><circle cx="5" cy="7.5" r="0.8" fill="#fff"></circle></svg>{alert}</span>') if alert else ""
            bars += (f'<div style="position: absolute; left: {left:.2f}%; width: {right - left:.2f}%; top: {lane * LANE_H + 8}px; height: 26px; border-radius: 9px; overflow: hidden; display: flex; box-shadow: 0 1px 0 rgba(31,58,46,0.12)">'
                     f'{segs}<div style="position: absolute; inset: 0; display: flex; align-items: center; padding: 0 9px; font-size: 12px; font-weight: 600; color: {C["ink"]}; white-space: nowrap; overflow: hidden">{nm}{badge}</div></div>')
        if pid == "A":
            l = pct("06-07")
            r = (doy("11-15") + 1) / DAYS * 100
            bars += (f'<a href="E07_planting_mobile.dc.html" style="text-decoration: none; position: absolute; left: {l:.2f}%; width: {r - l:.2f}%; top: 8px; height: 26px; border-radius: 9px; border: 2px dashed {C["accent"]}; box-sizing: border-box; display: flex; align-items: center; gap: 6px; padding: 0 10px; font-size: 12px; color: {C["accentInk"]}; white-space: nowrap; overflow: hidden; background: #FFF8EF">'
                     f'<strong>Free slot · 162 d</strong><span>· leek, carrot or tomato?</span></a>')
        tracks.append(f'<div style="position: relative; height: {hgt}px; border-top: 1px solid {C["line"]}; {"background: rgba(242,166,90,0.07);" if sel else ""}">{bars}</div>')
    month_lines = ""
    acc = 0
    for d in MDAYS[:-1]:
        acc += d
        month_lines += f'<div style="position: absolute; top: 0; bottom: 0; left: {acc / DAYS * 100:.2f}%; width: 1px; background: {C["line"]}"></div>'
    overlay = (f'{month_lines}'
               + annotations() +
               f'<div style="position: absolute; top: -30px; bottom: 0; left: {cur:.2f}%; width: 2px; margin-left: -1px; background: {C["accent"]}"></div>'
               f'<div style="position: absolute; top: -36px; left: {cur:.2f}%; transform: translateX(-50%); padding: 3px 9px; border-radius: 99px; background: {C["accent"]}; color: {C["accentInk"]}; font-size: 11px; font-weight: 700; white-space: nowrap">20 Jun</div>')
    return (f'<div style="overflow-x: auto"><div style="min-width: 980px; display: grid; grid-template-columns: 150px minmax(0, 1fr)">'
            f'<div></div><div style="display: flex; justify-content: space-between; align-items: center; padding: 0 0 8px; font-size: 13px">'
            f'<span style="display: inline-flex; align-items: center; gap: 6px; color: {C["muted"]}">{icon("back", 14, C["muted"])}Dec 2026</span>'
            f'<strong style="{H}; font-size: 18px">2027</strong>'
            f'<span style="display: inline-flex; align-items: center; gap: 6px; color: {C["muted"]}">Jan 2028<span style="display: inline-flex; transform: scaleX(-1)">{icon("back", 14, C["muted"])}</span></span></div>'
            f'<div style="padding-bottom: 8px"></div><div style="display: grid; grid-template-columns: {cols}">{months}</div>'
            f'<div style="display: flex; flex-direction: column; border-bottom: 1px solid {C["line"]}">{"".join(labels)}</div>'
            f'<div style="position: relative; border-bottom: 1px solid {C["line"]}">{overlay}<div style="position: relative">{"".join(tracks)}</div>'
            f'<div aria-hidden="true" style="position: absolute; top: 0; bottom: 0; left: 0; width: 28px; background: linear-gradient(90deg, #FFFFFF, rgba(255,255,255,0)); pointer-events: none; z-index: 3"></div>'
            f'<div aria-hidden="true" style="position: absolute; top: 0; bottom: 0; right: 0; width: 28px; background: linear-gradient(270deg, #FFFFFF, rgba(255,255,255,0)); pointer-events: none; z-index: 3"></div></div>'
            f'</div></div>')


# ---------- interface elements ----------
FONTS = '<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link href="https://fonts.googleapis.com/css2?family=Fredoka:wght@500;600&amp;family=Figtree:wght@400;500;600;700&amp;display=swap" rel="stylesheet">'

LOGO = (f'<svg width="34" height="34" viewBox="0 0 34 34" aria-hidden="true">'
        f'{poly([(17, 4), (6, 12), (7, 22), (17, 30)], "#6BBA62")}{poly([(17, 4), (28, 12), (27, 22), (17, 30)], "#4A9450")}'
        f'{poly([(17, 4), (11, 10), (17, 15), (23, 10)], "#8ED17A")}<circle cx="21" cy="21" r="4" fill="#E5533D"></circle><circle cx="19.8" cy="19.8" r="1.2" fill="#fff" fill-opacity="0.6"></circle></svg>')


def icon(name, size=18, color=None):
    color = color or C["ink"]
    paths = {
        "play": f'<path d="M7 4.5v11l9-5.5z" fill="{color}"></path>',
        "plus": f'<path d="M10 4v12M4 10h12" stroke="{color}" stroke-width="2" stroke-linecap="round" fill="none"></path>',
        "minus": f'<path d="M4 10h12" stroke="{color}" stroke-width="2" stroke-linecap="round" fill="none"></path>',
        "rotate": f'<path d="M15.5 8A6 6 0 1 0 16 12" stroke="{color}" stroke-width="2" stroke-linecap="round" fill="none"></path><path d="M16 3.5V8h-4.5" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "check": f'<path d="M4.5 10.5l3.5 3.5 7.5-8" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "down": f'<path d="M5.5 8l4.5 4.5L14.5 8" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "export": f'<path d="M10 3v9M6.5 6.5L10 3l3.5 3.5M4 12.5V16h12v-3.5" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "sun": f'<circle cx="10" cy="10" r="3.5" fill="none" stroke="{color}" stroke-width="1.8"></circle><path d="M10 2.5v2M10 15.5v2M2.5 10h2M15.5 10h2M4.7 4.7l1.4 1.4M13.9 13.9l1.4 1.4M4.7 15.3l1.4-1.4M13.9 6.1l1.4-1.4" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
        "leaf": f'<path d="M4 16c0-7 4-11 12-12-1 8-5 12-12 12zM4 16l6-6" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "garden": f'<path d="M3 12l7-4 7 4-7 4z" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" fill="none"></path><path d="M10 8V4.5M8 6l2-1.5L12 6" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "bars": f'<path d="M3 5h8M6 10h10M3 15h6" stroke="{color}" stroke-width="2" stroke-linecap="round" fill="none"></path>',
        "bulb": f'<path d="M10 3a5 5 0 0 0-3 9v2h6v-2a5 5 0 0 0-3-9zM8 17h4" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "book": f'<path d="M4 4h5a2 2 0 0 1 2 2v10a2 2 0 0 0-2-2H4zM16 4h-5" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" stroke-linecap="round" fill="none"></path><path d="M16 4v10h-5" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" fill="none"></path>',
        "close": f'<path d="M5 5l10 10M15 5L5 15" stroke="{color}" stroke-width="2" stroke-linecap="round" fill="none"></path>',
        "cal": f'<rect x="3" y="4.5" width="14" height="12" rx="2.5" fill="none" stroke="{color}" stroke-width="1.8"></rect><path d="M3 8.5h14M7 3v3M13 3v3" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
        "edit": f'<path d="M4 16l1-4 8.5-8.5a2 2 0 0 1 3 3L8 15z" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" fill="none"></path><path d="M12 5l3 3" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
        "move": f'<path d="M10 2.5v15M2.5 10h15M10 2.5l-2.5 2.5M10 2.5l2.5 2.5M10 17.5l-2.5-2.5M10 17.5l2.5-2.5M2.5 10L5 7.5M2.5 10L5 12.5M17.5 10L15 7.5M17.5 10L15 12.5" stroke="{color}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "rect": f'<rect x="3" y="6" width="14" height="8" rx="1.5" fill="none" stroke="{color}" stroke-width="1.8"></rect>',
        "square": f'<rect x="5" y="5" width="10" height="10" rx="1.5" fill="none" stroke="{color}" stroke-width="1.8"></rect>',
        "tree": f'<path d="M10 2.5l5 7h-2.5l3 4.5h-11l3-4.5H5z" stroke="{color}" stroke-width="1.6" stroke-linejoin="round" fill="none"></path><path d="M10 14v3.5" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
        "deco": f'<circle cx="10" cy="11" r="5.5" fill="none" stroke="{color}" stroke-width="1.8"></circle><path d="M10 5.5V3" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
        "copy": f'<rect x="7" y="7" width="10" height="10" rx="2" fill="none" stroke="{color}" stroke-width="1.8"></rect><path d="M13 4H5a1 1 0 0 0-1 1v8" stroke="{color}" stroke-width="1.8" stroke-linecap="round" fill="none"></path>',
        "trash": f'<path d="M4.5 6h11M8 6V4h4v2M6 6l.8 10h6.4L14 6" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "undo": f'<path d="M7 5L3.5 8.5 7 12M4 8.5h8a4 4 0 0 1 0 8H9" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "redo": f'<path d="M13 5l3.5 3.5L13 12M16 8.5H8a4 4 0 0 0 0 8h3" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "back": f'<path d="M12 4.5L6.5 10l5.5 5.5" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "frost": f'<path d="M10 2.5v15M3.5 6.2l13 7.6M3.5 13.8l13-7.6" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
    }[name]
    return f'<svg width="{size}" height="{size}" viewBox="0 0 20 20" aria-hidden="true">{paths}</svg>'


def crop_icon(kind, stage=2, size=44):
    k = size / 44
    return (f'<svg width="{size}" height="{size}" viewBox="0 0 {size} {size}" aria-hidden="true">'
            f'<ellipse cx="{size / 2:.1f}" cy="{size * 0.8:.1f}" rx="{size * 0.36:.1f}" ry="{size * 0.15:.1f}" fill="{C["bedTop"]}"></ellipse>'
            f'{plant(kind, stage, size / 2, size * 0.8, k * 0.82)}</svg>')


def scrubber(compact=False):
    cur = pct("06-20")
    ticks = "".join(f'<span style="flex: {d} 1 0; font-size: 11px; color: {C["muted"]}; text-align: left; padding-left: 4px; border-left: 1px solid {C["line"]}">{m[:1] if compact else m}</span>' for m, d in zip(MONTHS, MDAYS))
    return (f'<div style="display: flex; align-items: center; gap: {12 if compact else 18}px">'
            f'<button aria-label="Play the season" style="flex: none; width: {48 if compact else 56}px; height: {48 if compact else 56}px; border-radius: 50%; border: 0; background: {C["accent"]}; display: grid; place-items: center; box-shadow: 0 6px 16px rgba(242,166,90,0.35); cursor: pointer">{icon("play", 22, C["accentInk"])}</button>'
            f'<button aria-label="Previous week" style="flex: none; width: 40px; height: 40px; border-radius: 50%; border: 1px solid {C["line"]}; background: #FFFFFF; display: grid; place-items: center; cursor: pointer">{icon("back", 16)}</button>'
            f'<div style="flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; gap: 8px">'
            f'<div style="position: relative; height: 10px; border-radius: 99px; background: #FFFFFF">'
            f'<div style="position: absolute; left: 0; top: 0; bottom: 0; width: {cur:.1f}%; border-radius: 99px; background: {mix(C["accent"], 0.35)}"></div>'
            f'<div style="position: absolute; top: 50%; left: {cur:.1f}%; width: 22px; height: 22px; margin: -11px 0 0 -11px; border-radius: 50%; background: #FFFFFF; border: 4px solid {C["accent"]}; box-sizing: border-box"></div></div>'
            f'<div style="display: flex">{ticks}</div></div>'
            f'<button aria-label="Next week" style="flex: none; width: 40px; height: 40px; border-radius: 50%; border: 1px solid {C["line"]}; background: #FFFFFF; display: grid; place-items: center; cursor: pointer"><span style="display: inline-flex; transform: scaleX(-1)">{icon("back", 16)}</span></button></div>')


def chip(text, bg="#FFFFFF", fg=None, icon_html=""):
    return f'<span style="display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 99px; background: {bg}; color: {fg or C["ink"]}; font-size: 13px; font-weight: 600">{icon_html}{text}</span>'


def page(title, body, props, lang="fr"):
    return f'''<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<title>{title}</title>
<script src="./support.js"></script>
</head>
<body>
<x-dc>
<helmet>
{FONTS}
<style>
body{{margin:0;background:{C["bg"]}}}
a{{color:{C["green"]}}}a:hover{{color:{C["ink"]}}}
</style>
</helmet>
{body}
</x-dc>
<script type="text/x-dc" data-dc-script data-props='{json.dumps(props, ensure_ascii=False)}'>
class Component extends DCLogic {{
renderVals() {{
return {{}};
}}
}}
</script>
</body>
</html>
'''


ROOTSTYLE = f'font-family: Figtree, system-ui, sans-serif; color: {C["ink"]}; background: {C["bg"]}'
H = "font-family: Fredoka, Figtree, sans-serif; font-weight: 600; margin: 0"


def mode_switch(active):
    items = (("garden", "Garden", "Main.dc.html"), ("edit", "Plan", "E03_plan-editor_desktop.dc.html"))
    out = ""
    for ic, t, href in items:
        on = t == active
        out += (f'<a href="{href}" aria-current="{"page" if on else "false"}" style="display: inline-flex; align-items: center; gap: 8px; height: 40px; padding: 0 18px; border-radius: 99px; text-decoration: none; font: 600 15px Figtree, sans-serif; '
                f'{"background: " + C["ink"] + "; color: #FFFFFF" if on else "background: transparent; color: " + C["ink"]}">{icon(ic, 18, "#FFFFFF" if on else C["ink"])}{t}</a>')
    return f'<nav aria-label="Mode" style="display: flex; gap: 2px; padding: 4px; border-radius: 99px; background: #FFFFFF; border: 1px solid {C["line"]}">{out}</nav>'


# ---------- Artboard 1: Garden screen (computer) ----------
def main_board():
    seasons = "".join(
        f'<button style="border: 0; padding: 8px 16px; border-radius: 99px; font: 600 14px Figtree, sans-serif; cursor: pointer; '
        f'{"background: " + C["ink"] + "; color: #FFFFFF" if y == 2027 else "background: transparent; color: " + C["muted"]}">{y}</button>'
        for y in (2026, 2027, 2028))
    header = (
        f'<header style="display: flex; flex-wrap: wrap; align-items: center; gap: 14px 20px; padding: 20px 32px">'
        f'<div style="display: flex; align-items: center; gap: 10px; margin-right: 8px">{LOGO}<span style="{H}; font-size: 26px; color: {C["ink"]}">Potagerie</span></div>'
        f'{mode_switch("Garden")}'
        f'<button style="display: inline-flex; align-items: center; gap: 8px; border: 1px solid {C["line"]}; background: #FFFFFF; border-radius: 99px; padding: 9px 16px; font: 600 14px Figtree, sans-serif; color: {C["ink"]}; cursor: pointer">{icon("cal", 16)}Today</button>'
        f'<div style="flex: 1 1 40px"></div>'
        f'<span style="display: inline-flex; align-items: center; gap: 6px; font-size: 13px; color: {C["muted"]}">{icon("check", 16, C["good"])}Saved</span>'
        f'</header>')
    ctrl = (f'<button style="display: inline-flex; align-items: center; gap: 8px; height: 42px; padding: 0 16px; border-radius: 99px; border: 0; background: #FFFFFF; color: {C["ink"]}; font: 600 14px Figtree, sans-serif; cursor: pointer; box-shadow: 0 2px 6px rgba(31,58,46,0.08)">'
            f'{icon("rotate", 17)}Rotate the view</button>')

    stage = (
        f'<section style="flex: 999 1 640px; min-width: 0; background: {C["card"]}; border-radius: 32px; padding: 24px 28px 26px; display: flex; flex-direction: column; gap: 18px">'
        f'<div style="display: flex; flex-wrap: wrap; justify-content: space-between; align-items: flex-start; gap: 12px">'
        f'<div style="display: flex; flex-direction: column; gap: 6px"><span style="{H}; font-size: 34px">20 Jun 2027</span>'
        f'<div style="display: flex; gap: 8px; flex-wrap: wrap">{chip("Summer", "#FFFFFF")}{chip("7 plantings in the ground")}{chip("1 free bed", "#FFF3E4", C["accentInk"])}</div></div>'
        f'<div style="display: flex; gap: 8px">{ctrl}</div></div>'
        f'{diorama(1.0, 960, 580)}'
        f'{scrubber()}'
        f'</section>')
    series = [
        ("tomate", "Tomato", "Half of the bed · ≈ 3 plants (1 row × 3)", "Sowing under cover 10 Mar · In the ground 5 May · Harvest 19 Jul → 27 Sep", "Adult"),
        ("laitue", "Lettuce", "Half of the bed · ≈ 18 plants (3 rows × 6)", "Sowing under cover 15 Apr · In the ground 13 May · Harvest 2 Jul → 22 Jul", "Adult"),
    ]
    srows = "".join(
        f'<div style="display: flex; gap: 12px; align-items: flex-start; padding: 12px; border-radius: 18px; background: {C["bg"]}">'
        f'<span style="flex: none; width: 52px; height: 52px; border-radius: 14px; background: #FFFFFF; display: grid; place-items: center">{crop_icon(k, 2, 44)}</span>'
        f'<div style="display: flex; flex-direction: column; gap: 3px; min-width: 0"><div style="display: flex; align-items: center; gap: 8px"><strong style="font-size: 15px">{n}</strong>'
        f'<span style="font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 99px; background: #FFFFFF; color: {C["green"]}">{st}</span></div>'
        f'<span style="font-size: 13px; color: {C["muted"]}">{q}</span><span style="font-size: 13px; color: {C["ink"]}">{d}</span></div></div>'
        for k, n, q, d, st in series)
    aside = (
        f'<aside style="flex: 1 1 320px; max-width: 400px; min-width: 0; background: #FFFFFF; border-radius: 28px; padding: 24px; display: flex; flex-direction: column; gap: 18px">'
        f'<div style="display: flex; justify-content: space-between; align-items: center; gap: 8px"><label for="pl-name" style="flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Bed B · name</span><input id="pl-name" value="The tomatoes" style="{H}; font-size: 24px; color: {C["ink"]}; border: 0; border-bottom: 2px dashed {C["line"]}; background: transparent; padding: 2px 0; width: 100%; min-width: 0"></label>'
        f'<button aria-label="Close the panel" style="width: 36px; height: 36px; border-radius: 50%; border: 0; background: {C["bg"]}; display: grid; place-items: center; cursor: pointer">{icon("close", 16)}</button></div>'
        f'<div style="display: flex; gap: 8px; flex-wrap: wrap">{chip("4 × 1 m", C["bg"])}{chip("4 m²", C["bg"])}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 10px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">In the ground on 20 Jun</span>{srows}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 8px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Notes on the timeline</span>'
        f'<div style="display: flex; gap: 10px; align-items: flex-start; padding: 12px 14px; border-radius: 16px; background: #FFF1BF; color: #5A470A; font-size: 14px">{icon("cal", 18, "#8A6D0E")}<span><strong>1 Jul</strong> · Mulch the tomatoes before the heat.</span></div>'
        f'<button style="align-self: flex-start; display: inline-flex; align-items: center; gap: 6px; border: 0; background: transparent; padding: 4px 0; font: 600 14px Figtree, sans-serif; color: {C["green"]}; cursor: pointer">{icon("plus", 16, C["green"])}Add a note</button></div>'
        f'<div style="display: flex; flex-direction: column; gap: 6px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Before on this bed</span>'
        f'<div style="display: flex; justify-content: space-between; font-size: 14px"><span>2026</span><span style="color: {C["muted"]}">Apiaceae (carrot)</span></div>'
        f'<div style="display: flex; justify-content: space-between; font-size: 14px"><span>2025</span><span style="color: {C["muted"]}">Nothing noted</span></div></div>'
        f'<div style="display: flex; gap: 10px; flex-wrap: wrap; margin-top: auto">'
        f'<a href="E07_planting_mobile.dc.html" style="flex: 1 1 140px; display: inline-flex; justify-content: center; align-items: center; gap: 8px; background: {C["green"]}; color: #FFFFFF; text-decoration: none; border-radius: 99px; padding: 12px 16px; font: 600 15px Figtree, sans-serif">{icon("plus", 18, "#FFFFFF")}Plant here</a>'
        f'<a href="../../POT-D02_ecrans-complementaires_v0.1/project/E06_bed-panel_desktop.dc.html" style="flex: 1 1 120px; display: inline-flex; justify-content: center; align-items: center; border: 1px solid {C["line"]}; background: #FFFFFF; color: {C["ink"]}; text-decoration: none; border-radius: 99px; padding: 12px 16px; font: 600 15px Figtree, sans-serif; cursor: pointer">Edit</a></div>'
        f'</aside>')
    legend = "".join(
        f'<span style="display: inline-flex; align-items: center; gap: 8px; font-size: 13px; color: {C["muted"]}"><span style="width: 26px; height: 12px; border-radius: 4px; {bg}"></span>{t}</span>'
        for bg, t in ((f"background: repeating-linear-gradient(135deg, {mix('#E5533D', 0.55)} 0 4px, {mix('#E5533D', 0.85)} 4px 8px)", "Sowing under cover"),
                      (f"background: {mix('#E5533D', 0.45)}", "Growth"), ("background: #E5533D", "Harvest"),
                      ("background: #ECE6FA; border: 1px dashed #8B78D0", "Annotated period")))
    frise = (
        f'<section style="margin: 24px 32px 32px; background: #FFFFFF; border-radius: 28px; padding: 22px 24px 26px; display: flex; flex-direction: column; gap: 40px">'
        f'<div style="display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 12px 24px">'
        f'<div style="display: flex; flex-direction: column; gap: 2px"><h2 style="{H}; font-size: 22px">Crop timeline</h2><span style="font-size: 13px; color: {C["muted"]}">Drag the timeline to go from one year to the next, without a break.</span></div>'
        f'<div style="display: flex; flex-wrap: wrap; gap: 10px 18px; align-items: center">{legend}'
        f'<button style="display: inline-flex; align-items: center; gap: 6px; border: 0; background: #FFF1BF; border-radius: 99px; padding: 8px 14px; font: 600 13px Figtree, sans-serif; color: #5A470A; cursor: pointer">{icon("plus", 14, "#5A470A")}Annotate</button>'
        f'<button style="display: inline-flex; align-items: center; gap: 6px; border: 0; background: {C["green"]}; border-radius: 99px; padding: 8px 14px; font: 600 13px Figtree, sans-serif; color: #FFFFFF; cursor: pointer">{icon("export", 14, "#FFFFFF")}Export the timeline</button>'
        f'<button style="display: inline-flex; align-items: center; gap: 6px; border: 1px solid {C["line"]}; background: #FFFFFF; border-radius: 99px; padding: 7px 12px; font: 600 13px Figtree, sans-serif; color: {C["ink"]}; cursor: pointer">Replier {icon("down", 14)}</button></div></div>'
        f'{gantt()}</section>')
    body = (f'<div style="{ROOTSTYLE}; min-height: 1560px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px; align-items: stretch">{stage}{aside}</div>'
            f'{frise}</div>')
    return page("Garden — computer", body, {"$preview": {"width": 1440, "height": 1560}})


# ---------- Artboard 2: Garden screen (phone) ----------
def mobile_board():
    tabs = "".join(
        f'<a href="#" style="flex: 1; display: flex; flex-direction: column; align-items: center; gap: 4px; text-decoration: none; font-size: 11px; font-weight: 600; color: {C["ink"] if on else C["muted"]}">'
        f'<span style="width: 52px; height: 30px; border-radius: 99px; display: grid; place-items: center; {"background: " + C["card"] if on else ""}">{icon(ic, 20, C["ink"] if on else C["muted"])}</span>{t}</a>'
        for ic, t, on in (("garden", "Garden", True), ("bars", "Timeline", False), ("book", "Catalogue", False)))
    body = (
        f'<div style="{ROOTSTYLE}; width: 390px; height: 844px; box-sizing: border-box; display: flex; flex-direction: column; overflow: hidden">'
        f'<header style="display: flex; align-items: center; justify-content: space-between; padding: 18px 20px 8px">'
        f'<div style="display: flex; align-items: center; gap: 8px">{LOGO}<span style="{H}; font-size: 22px">Potagerie</span></div>'
        f'<button style="display: inline-flex; align-items: center; gap: 4px; border: 1px solid {C["line"]}; background: #FFFFFF; border-radius: 99px; padding: 8px 12px; font: 600 14px Figtree, sans-serif; color: {C["ink"]}">{icon("cal", 14)}Today</button></header>'
        f'<div style="padding: 4px 20px 0; display: flex; flex-direction: column; gap: 6px"><span style="{H}; font-size: 28px">20 Jun</span>'
        f'<div style="display: flex; gap: 6px">{chip("Summer", "#FFFFFF")}{chip("1 note", "#FFF1BF", "#5A470A")}</div></div>'
        f'<div style="flex: 1 1 auto; display: flex; align-items: center; padding: 0 4px">{diorama(0.43, 382, 270, ox=13 * 24 * 0.43 + 12, oy=58, labels="short")}</div>'
        f'<div style="background: #FFFFFF; border-radius: 28px 28px 0 0; padding: 14px 20px 10px; display: flex; flex-direction: column; gap: 14px; box-shadow: 0 -6px 20px rgba(31,58,46,0.06)">'
        f'<span aria-hidden="true" style="align-self: center; width: 40px; height: 5px; border-radius: 99px; background: {C["line"]}"></span>'
        f'{scrubber(True)}'
        f'<div style="display: flex; gap: 12px; align-items: center; padding: 12px; border-radius: 18px; background: {C["bg"]}">'
        f'<span style="flex: none; width: 48px; height: 48px; border-radius: 14px; background: #FFFFFF; display: grid; place-items: center">{crop_icon("tomate", 2, 40)}</span>'
        f'<div style="display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0"><strong style="font-size: 15px">The tomatoes</strong><span style="font-size: 13px; color: {C["muted"]}">Tomato and lettuce · harvest from 2 Jul</span></div>'
        f'<a href="E07_planting_mobile.dc.html" aria-label="Plant on The tomatoes" style="flex: none; width: 44px; height: 44px; border-radius: 50%; background: {C["green"]}; display: grid; place-items: center">{icon("plus", 20, "#FFFFFF")}</a></div>'
        f'<nav style="display: flex; padding-top: 6px; border-top: 1px solid {C["line"]}">{tabs}</nav></div>'
        f'</div>')
    return page("Garden — phone", body, {"$preview": {"width": 390, "height": 844}})


# ---------- Artboard 3: new planting (phone) ----------
def serie_board():
    W, HGT = 390, 1180
    lab = lambda n, t: (f'<div style="display: flex; align-items: center; gap: 8px"><span style="width: 22px; height: 22px; border-radius: 50%; background: {C["ink"]}; color: #FFFFFF; display: grid; place-items: center; font: 700 12px Figtree, sans-serif">{n}</span>'
                        f'<span style="font: 600 16px Fredoka, Figtree, sans-serif; color: {C["ink"]}">{t}</span></div>')
    crops = [("poireau", "Leek", True), ("laitue", "Lettuce", False), ("carotte", "Carrot", False),
             ("haricot", "Bean", False), ("courgette", "Zucchini", False), ("tomate", "Tomato", False)]
    grid = "".join(
        f'<button aria-pressed="{"true" if sel else "false"}" style="display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 8px 2px 6px; border-radius: 16px; cursor: pointer; '
        f'border: {"2px solid " + C["accent"] if sel else "2px solid transparent"}; background: {"#FFF3E4" if sel else C["bg"]}; font: 600 11px Figtree, sans-serif; color: {C["ink"]}">'
        f'{crop_icon(k, 2, 38)}{n}</button>'
        for k, n, sel in crops)
    # icons of the start modes
    ic_godet = ('<svg width="40" height="40" viewBox="0 0 40 40" aria-hidden="true"><path d="M11 22h18l-3 13H14z" fill="#C9875A"></path><path d="M11 22h18v3H11z" fill="#B0703F"></path>'
                '<path d="M20 22v-7" stroke="#4C8F48" stroke-width="2" stroke-linecap="round"></path><ellipse cx="16" cy="14" rx="4" ry="2.2" fill="#7CC46A" transform="rotate(-25 16 14)"></ellipse><ellipse cx="24" cy="13" rx="4" ry="2.2" fill="#5FAA55" transform="rotate(25 24 13)"></ellipse></svg>')
    ic_plant = ('<svg width="40" height="40" viewBox="0 0 40 40" aria-hidden="true"><path d="M12 25h16l-2 10H14z" fill="#3E6E52"></path>'
                '<path d="M20 25V12" stroke="#4C8F48" stroke-width="2" stroke-linecap="round"></path><ellipse cx="14" cy="17" rx="6" ry="3" fill="#6CBB62" transform="rotate(-30 14 17)"></ellipse><ellipse cx="26" cy="15" rx="6" ry="3" fill="#4C944D" transform="rotate(30 26 15)"></ellipse>'
                '<ellipse cx="20" cy="10" rx="4" ry="2.5" fill="#86CB74"></ellipse><rect x="27" y="27" width="10" height="7" rx="2" fill="#F2A65A"></rect></svg>')
    ic_seed = ('<svg width="40" height="40" viewBox="0 0 40 40" aria-hidden="true"><path d="M4 28q16-5 32 0v6H4z" fill="#A0704A"></path>'
               '<ellipse cx="13" cy="20" rx="2.6" ry="1.6" fill="#8A6A3A" transform="rotate(-20 13 20)"></ellipse><ellipse cx="21" cy="16" rx="2.6" ry="1.6" fill="#8A6A3A"></ellipse><ellipse cx="28" cy="21" rx="2.6" ry="1.6" fill="#8A6A3A" transform="rotate(25 28 21)"></ellipse>'
               '<path d="M13 23v3M21 19v6M28 24v3" stroke="#8A6A3A" stroke-width="1" stroke-dasharray="1.5 1.5"></path></svg>')
    modes = [(ic_godet, "I sow under cover", "In pots, then transplanted to the garden", True, True),
             (ic_plant, "I buy plants", "Ready to plant, from a garden centre", False, True),
             (ic_seed, "I sow in the open ground", "Directly in the bed", False, True)]
    mode_cards = "".join(
        f'<button aria-pressed="{"true" if sel else "false"}" {"" if ok else "disabled"} style="display: flex; align-items: center; gap: 12px; width: 100%; text-align: left; padding: 10px 12px; border-radius: 16px; cursor: {"pointer" if ok else "not-allowed"}; '
        f'border: {"2px solid " + C["accent"] if sel else "2px solid " + C["line"]}; background: {"#FFF3E4" if sel else ("#FFFFFF" if ok else C["bg"])}; opacity: {1 if ok else 0.55}; font: 600 15px Figtree, sans-serif; color: {C["ink"]}">'
        f'<span style="flex: none; width: 44px; height: 44px; border-radius: 12px; background: {"#FFFFFF" if sel else C["bg"]}; display: grid; place-items: center">{ic}</span>'
        f'<span style="display: flex; flex-direction: column; gap: 1px; flex: 1"><span>{t}</span><span style="font-size: 12px; font-weight: 500; color: {C["muted"]}">{sub}</span></span>'
        f'<span aria-hidden="true" style="flex: none; width: 20px; height: 20px; border-radius: 50%; border: 2px solid {C["accent"] if sel else C["line"]}; box-sizing: border-box; display: grid; place-items: center">{"<span style=" + chr(34) + "width: 10px; height: 10px; border-radius: 50%; background: " + C["accent"] + chr(34) + "></span>" if sel else ""}</span></button>'
        for ic, t, sub, sel, ok in modes)
    # mini-frise : 6 Apr -> 11 Jan
    tot = 280.0
    segs = [(70, "repeating-linear-gradient(135deg, #B7DAC4 0 4px, #E2F0E7 4px 8px)", "Godets"), (120, "#B7DAC4", "En terre"), (90, "#6FB487", "Harvest")]
    bar = "".join(f'<div style="flex: {d} 1 0; height: 100%; background: {bg}"></div>' for d, bg, _ in segs)
    keys = [("6 Apr", "Sowing under cover", 0), ("15 Jun", "Transplanting", 70 / tot * 100), ("13 Oct", "First harvest", 190 / tot * 100), ("11 Jan", "End", 100)]
    marks = "".join(
        f'<div style="position: absolute; left: {x:.1f}%; top: 0; transform: translateX({"0" if x == 0 else ("-100%" if x == 100 else "-50%")}); display: flex; flex-direction: column; align-items: {"flex-start" if x == 0 else ("flex-end" if x == 100 else "center")}; white-space: nowrap">'
        f'<strong style="font-size: 13px">{d}</strong><span style="font-size: 11px; color: {C["muted"]}">{t}</span></div>'
        for d, t, x in keys)
    timeline = (f'<div style="display: flex; flex-direction: column; gap: 8px; padding: 14px; border-radius: 18px; background: {C["bg"]}">'
                f'<div style="display: flex; height: 14px; border-radius: 7px; overflow: hidden">{bar}</div>'
                f'<div style="position: relative; height: 34px">{marks}</div>'
                f'<div style="display: flex; justify-content: space-between; padding-top: 8px; border-top: 1px solid {C["line"]}; font-size: 13px"><span style="color: {C["muted"]}">≈ 48 plants · 3 rows × 16</span><strong>1 packet of seeds</strong></div></div>')
    parts = "".join(
        f'<button aria-pressed="{"true" if on else "false"}" style="flex: 1; border: 0; padding: 7px 0; border-radius: 99px; font: 600 13px Figtree, sans-serif; cursor: pointer; '
        f'{"background: " + C["ink"] + "; color: #FFFFFF" if on else "background: transparent; color: " + C["muted"]}">{t}</button>'
        for t, on in (("All", False), ("½", True), ("⅓", False), ("¼", False)))
    tsegs = "".join(
        f'<button aria-pressed="{"true" if i >= 4 else "false"}" aria-label="Segment {i + 1}" style="flex: 1; height: 36px; border: 0; cursor: pointer; '
        f'background: {"#B9DCC6" if i >= 4 else C["bedTop"]}; {"box-shadow: inset 0 0 0 2px " + C["green"] + ";" if i >= 4 else ""} border-radius: {"9px 0 0 9px" if i == 0 else ("0 9px 9px 0" if i == 7 else "0")}"></button>'
        for i in range(8))
    zone = (f'<div style="display: flex; flex-direction: column; gap: 8px">'
            f'<div style="display: flex; justify-content: space-between; align-items: baseline"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">Zone of the bed</span><span style="font-size: 13px; font-weight: 600">2 m by 4 m</span></div>'
            f'<div role="group" aria-label="Zone shortcuts" style="display: flex; padding: 4px; border-radius: 99px; background: {C["bg"]}">{parts}</div>'
            f'<div role="group" aria-label="50 cm segments" style="display: flex; gap: 3px">{tsegs}</div></div>')
    body = (
        f'<div style="{ROOTSTYLE}; width: {W}px; height: {HGT}px; box-sizing: border-box; background: rgba(31,58,46,0.35); display: flex; flex-direction: column; justify-content: flex-end">'
        f'<div style="background: #FFFFFF; border-radius: 28px 28px 0 0; padding: 12px 20px 22px; display: flex; flex-direction: column; gap: 16px">'
        f'<span aria-hidden="true" style="align-self: center; width: 40px; height: 5px; border-radius: 99px; background: {C["line"]}"></span>'
        f'<div style="display: flex; justify-content: space-between; align-items: center"><div style="display: flex; flex-direction: column; gap: 2px"><span style="font-size: 13px; color: {C["muted"]}">Free slot · 7 Jun → 15 Nov</span><h2 style="{H}; font-size: 24px">Plant on Back of the garden</h2></div>'
        f'<a href="E05_garden_mobile.dc.html" aria-label="Close" style="width: 38px; height: 38px; border-radius: 50%; background: {C["bg"]}; display: grid; place-items: center">{icon("close", 16)}</a></div>'
        f'{lab(1, "Which crop?")}'
        f'<div style="display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: 5px">{grid}</div>'
        f'{lab(2, "How do you start?")}'
        f'<div role="radiogroup" aria-label="Start mode" style="display: flex; flex-direction: column; gap: 8px">{mode_cards}</div>'
        f'{lab(3, "When do you sow?")}'
        f'<label for="d-semis" style="display: flex; align-items: center; gap: 10px; padding: 0 14px; height: 50px; border-radius: 14px; background: {C["bg"]}">{icon("cal", 20, C["muted"])}'
        f'<input id="d-semis" value="Mardi 6 April 2027" style="border: 0; background: transparent; font: 600 16px Figtree, sans-serif; color: {C["ink"]}; width: 100%; min-width: 0"></label>'
        f'<a href="#" style="align-self: flex-start; margin-top: -8px; font-size: 13px; font-weight: 600">I rather know the transplanting date</a>'
        f'{timeline}'
        f'{zone}'
        f'<a href="E05_garden_mobile.dc.html" style="display: block; text-align: center; background: {C["green"]}; color: #FFFFFF; text-decoration: none; border-radius: 99px; padding: 15px; font: 600 16px Figtree, sans-serif">Plant 48 leeks</a>'
        f'</div></div>')
    return page("New planting — phone", body, {"$preview": {"width": W, "height": HGT}})


# ---------- Artboard 4 : visual direction ----------
def style_board():
    sw = [("Sage background", C["bg"]), ("Map", C["card"]), ("Ink", C["ink"]), ("Soft text", C["muted"]),
          ("Foliage", C["green"]), ("Apricot", C["accent"]), ("Soil", C["bedTop"]), ("Wood", C["woodTop"])]
    swatches = "".join(
        f'<div style="display: flex; flex-direction: column; gap: 8px"><span style="height: 72px; border-radius: 18px; background: {v}; border: 1px solid {C["line"]}"></span>'
        f'<strong style="font-size: 14px">{n}</strong><span style="font-size: 13px; color: {C["muted"]}">{v}</span></div>' for n, v in sw)
    sem = "".join(f'<span style="display: inline-flex; align-items: center; gap: 8px; padding: 8px 14px; border-radius: 99px; background: {bg}; color: {fg}; font-size: 14px; font-weight: 600">'
                  f'<span style="width: 10px; height: 10px; border-radius: 50%; background: {dot}"></span>{t}</span>'
                  for t, bg, fg, dot in (("Note · 1 Jul", "#FFF1BF", "#5A470A", "#D9B21F"), ("Period · holidays", "#ECE6FA", "#4B3C7A", "#8B78D0"),
                                         ("Free slot", "#FFF3E4", C["accentInk"], C["accent"])))
    stage_names = ["Sowing", "Young shoot", "Adult", "Harvest"]
    kinds = [("tomate", "Tomato"), ("laitue", "Lettuce"), ("carotte", "Carrot"), ("haricot", "Bean"), ("courgette", "Zucchini"), ("poireau", "Leek")]
    head = "".join(f'<span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}; text-align: center">{s}</span>' for s in stage_names)
    rows = ""
    for k, n in kinds:
        cells = "".join(f'<span style="display: grid; place-items: center; height: 96px; border-radius: 20px; background: {C["card"]}">{crop_icon(k, st, 84)}</span>' for st in range(4))
        rows += f'<strong style="font-size: 15px; align-self: center">{n}</strong>{cells}'
    body = (
        f'<div style="{ROOTSTYLE}; width: 1440px; height: 1080px; box-sizing: border-box; padding: 64px; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 56px">'
        f'<div style="display: flex; flex-direction: column; gap: 36px">'
        f'<div style="display: flex; flex-direction: column; gap: 10px"><span style="font-size: 13px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: {C["muted"]}">Visual direction</span>'
        f'<h1 style="{H}; font-size: 56px; line-height: 1.05">A soft diorama,<br>that grows with the season.</h1>'
        f'<p style="margin: 0; font-size: 18px; line-height: 1.55; color: {C["muted"]}; max-width: 560px">Faceted isometric, desaturated garden colours, a white rounded interface around it. The garden fills the stage; the controls stay discreet.</p></div>'
        f'<div style="display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 16px">{swatches}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 14px"><span style="font-size: 13px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: {C["muted"]}">Typography</span>'
        f'<div style="display: flex; align-items: baseline; gap: 18px"><span style="{H}; font-size: 44px">Fredoka</span><span style="color: {C["muted"]}; font-size: 15px">Titles, dates, bed names</span></div>'
        f'<div style="display: flex; align-items: baseline; gap: 18px"><span style="font-size: 28px; font-weight: 600">Figtree</span><span style="color: {C["muted"]}; font-size: 15px">Interface, data, running texts</span></div></div>'
        f'<div style="display: flex; flex-direction: column; gap: 14px"><span style="font-size: 13px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: {C["muted"]}">Timeline annotations</span><div style="display: flex; flex-wrap: wrap; gap: 10px">{sem}</div></div>'
        f'<div style="display: flex; flex-wrap: wrap; gap: 12px">'
        f'<button style="border: 0; background: {C["green"]}; color: #FFFFFF; border-radius: 99px; padding: 13px 22px; font: 600 15px Figtree, sans-serif">Primary action</button>'
        f'<button style="border: 1px solid {C["line"]}; background: #FFFFFF; color: {C["ink"]}; border-radius: 99px; padding: 13px 22px; font: 600 15px Figtree, sans-serif">Secondary action</button>'
        f'<button aria-label="Play the season" style="width: 52px; height: 52px; border-radius: 50%; border: 0; background: {C["accent"]}; display: grid; place-items: center">{icon("play", 22, C["accentInk"])}</button></div>'
        f'</div>'
        f'<div style="display: flex; flex-direction: column; gap: 16px"><span style="font-size: 13px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: {C["muted"]}">The 4 growth stages</span>'
        f'<div style="display: grid; grid-template-columns: 110px repeat(4, minmax(0, 1fr)); gap: 12px">'
        f'<span></span>{head}{rows}</div>'
        f'<p style="margin: 0; font-size: 15px; line-height: 1.5; color: {C["muted"]}">Each crop has its recognisable silhouette at each stage. Between two stages, the plant grows with a slight bounce.</p></div>'
        f'</div>')
    return page("Visual direction", body, {"$preview": {"width": 1440, "height": 1080}})


# ---------- Artboard 5: plan editor (computer) ----------
def atelier_board():
    CELL = 40
    GW, GH = 22, 13
    W, Hh = GW * CELL, GH * CELL
    svg = [f'<rect x="0" y="0" width="{W}" height="{Hh}" rx="14" fill="{C["grassTop"]}"></rect>']
    # grille
    for i in range(1, GW):
        svg.append(f'<line x1="{i * CELL}" y1="0" x2="{i * CELL}" y2="{Hh}" stroke="#FFFFFF" stroke-opacity="{0.55 if i % 2 == 0 else 0.25}" stroke-width="1"></line>')
    for j in range(1, GH):
        svg.append(f'<line x1="0" y1="{j * CELL}" x2="{W}" y2="{j * CELL}" stroke="#FFFFFF" stroke-opacity="{0.55 if j % 2 == 0 else 0.25}" stroke-width="1"></line>')
    # planches
    for p in PLANCHES:
        x, y, w, h = p["x"] * CELL, p["y"] * CELL, p["w"] * CELL, p["h"] * CELL
        bac = p["kind"] == "bac"
        sel = p["id"] == "B"
        drag = p["id"] == "E"
        if drag:
            # original position, dotted
            svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="none" stroke="#FFFFFF" stroke-width="2" stroke-dasharray="7 6"></rect>')
            x += 1 * CELL
            svg.append(f'<rect x="{x + 4}" y="{y + 6}" width="{w}" height="{h}" rx="6" fill="#1F3A2E" fill-opacity="0.18"></rect>')
        svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="{C["woodTop"] if bac else C["bedTop"]}"></rect>')
        if bac:
            svg.append(f'<rect x="{x + 6}" y="{y + 6}" width="{w - 12}" height="{h - 12}" rx="3" fill="{C["bedTop"]}"></rect>')
        else:
            for k in range(1, int(p["h"] * 2)):
                yy = y + k * CELL / 2
                svg.append(f'<line x1="{x + 8}" y1="{yy}" x2="{x + w - 8}" y2="{yy}" stroke="{C["bedFurrow"]}" stroke-width="1.5" stroke-linecap="round"></line>')
        label = NAMES[p["id"]]
        lw = len(label) * 7.2 + 18
        svg.append(f'<rect x="{x + 8}" y="{y + 8}" width="{lw}" height="22" rx="11" fill="#FFFFFF" fill-opacity="0.92"></rect>'
                   f'<text x="{x + 8 + lw / 2}" y="{y + 23}" text-anchor="middle" font-family="Fredoka, sans-serif" font-weight="600" font-size="12.5" fill="{C["ink"]}">{label}</text>')
        if sel:
            svg.append(f'<rect x="{x - 3}" y="{y - 3}" width="{w + 6}" height="{h + 6}" rx="8" fill="none" stroke="{C["accent"]}" stroke-width="3"></rect>')
            for hx, hy in ((0, 0), (w / 2, 0), (w, 0), (0, h / 2), (w, h / 2), (0, h), (w / 2, h), (w, h)):
                svg.append(f'<rect x="{x + hx - 6}" y="{y + hy - 6}" width="12" height="12" rx="3" fill="#FFFFFF" stroke="{C["accent"]}" stroke-width="2.5"></rect>')
            svg.append(f'<text x="{x + w / 2}" y="{y - 12}" text-anchor="middle" font-family="Figtree, sans-serif" font-weight="700" font-size="12" fill="{C["accentInk"]}">4 m</text>'
                       f'<text x="{x + w + 14}" y="{y + h / 2 + 4}" font-family="Figtree, sans-serif" font-weight="700" font-size="12" fill="{C["accentInk"]}">1 m</text>')
        if drag:
            # guide d'alignement + cotes
            svg.append(f'<line x1="{x}" y1="{0.4 * CELL}" x2="{x}" y2="{12.6 * CELL}" stroke="{C["accent"]}" stroke-width="1.5" stroke-dasharray="5 5"></line>')
            gx1, gx2 = x + w, 22 * CELL
            svg.append(f'<line x1="{gx1 + 2}" y1="{y + h / 2}" x2="{gx2 - 2}" y2="{y + h / 2}" stroke="{C["ink"]}" stroke-width="1.5"></line>'
                       f'<rect x="{(gx1 + gx2) / 2 - 26}" y="{y + h / 2 - 12}" width="52" height="24" rx="12" fill="{C["ink"]}"></rect>'
                       f'<text x="{(gx1 + gx2) / 2}" y="{y + h / 2 + 4.5}" text-anchor="middle" font-family="Figtree, sans-serif" font-weight="700" font-size="12" fill="#FFFFFF">1.5 m</text>')
            svg.append(f'<line x1="{x + w / 2}" y1="{y + h + 2}" x2="{x + w / 2}" y2="{7 * CELL - 2}" stroke="{C["ink"]}" stroke-width="1.5"></line>'
                       f'<rect x="{x + w / 2 - 28}" y="{y + h + 8}" width="56" height="24" rx="12" fill="{C["ink"]}"></rect>'
                       f'<text x="{x + w / 2}" y="{y + h + 24.5}" text-anchor="middle" font-family="Figtree, sans-serif" font-weight="700" font-size="12" fill="#FFFFFF">50 cm</text>')
            svg.append(f'<g transform="translate({x + w / 2 - 14} {y + h / 2 - 14})"><circle cx="14" cy="14" r="16" fill="#FFFFFF"></circle>'
                       f'<path d="M14 4v20M4 14h20M14 4l-3 3M14 4l3 3M14 24l-3-3M14 24l3-3M4 14l3-3M4 14l3 3M24 14l-3-3M24 14l-3 3" stroke="{C["ink"]}" stroke-width="1.8" stroke-linecap="round" fill="none"></path></g>')
    # new bed being drawn
    nx, ny, nw, nh = 1 * CELL, 11 * CELL, 6 * CELL, 2 * CELL
    svg.append(f'<rect x="{nx}" y="{ny}" width="{nw}" height="{nh}" rx="6" fill="{C["bedTop"]}" fill-opacity="0.45" stroke="{C["green"]}" stroke-width="2.5" stroke-dasharray="8 5"></rect>'
               f'<rect x="{nx + nw / 2 - 46}" y="{ny + nh / 2 - 13}" width="92" height="26" rx="13" fill="{C["green"]}"></rect>'
               f'<text x="{nx + nw / 2}" y="{ny + nh / 2 + 4.5}" text-anchor="middle" font-family="Figtree, sans-serif" font-weight="700" font-size="12.5" fill="#FFFFFF">3 m × 1 m</text>'
               f'<circle cx="{nx + nw}" cy="{ny + nh}" r="7" fill="#FFFFFF" stroke="{C["green"]}" stroke-width="2.5"></circle>')
    # scale
    svg.append(f'<g transform="translate({W - 140} {Hh - 30})"><rect x="-10" y="-14" width="132" height="30" rx="15" fill="#FFFFFF" fill-opacity="0.92"></rect>'
               f'<line x1="0" y1="4" x2="{2 * CELL}" y2="4" stroke="{C["ink"]}" stroke-width="2.5"></line><line x1="0" y1="-1" x2="0" y2="9" stroke="{C["ink"]}" stroke-width="2"></line><line x1="{2 * CELL}" y1="-1" x2="{2 * CELL}" y2="9" stroke="{C["ink"]}" stroke-width="2"></line>'
               f'<text x="{2 * CELL + 10}" y="8" font-family="Figtree, sans-serif" font-weight="600" font-size="12" fill="{C["ink"]}">1 m</text></g>')
    plan_svg = (f'<svg viewBox="-20 -24 {W + 70} {Hh + 44}" width="100%" style="display: block; max-width: {W + 70}px" role="img" '
                f'aria-label="Plan seen from above, 11 × 6.5 m: seven beds, one bed being moved, a new bed dragged from the tray">{"".join(svg)}</svg>')

    # tray of items to drag onto the plot
    def tile_svg(kind):
        if kind == "planche":
            return f'<rect x="4" y="13" width="40" height="14" rx="3" fill="{C["bedTop"]}"></rect><line x1="8" y1="20" x2="40" y2="20" stroke="{C["bedFurrow"]}" stroke-width="1.5"></line>'
        if kind == "carre":
            return f'<rect x="10" y="6" width="28" height="28" rx="3" fill="{C["woodTop"]}"></rect><rect x="14" y="10" width="20" height="20" rx="2" fill="{C["bedTop"]}"></rect>'
        if kind == "bac":
            return f'<rect x="4" y="10" width="40" height="20" rx="3" fill="{C["woodTop"]}"></rect><rect x="8" y="14" width="32" height="12" rx="2" fill="{C["bedTop"]}"></rect>'
        if kind == "butte":
            return f'<path d="M4 28 Q24 8 44 28 Z" fill="{C["bedTop"]}"></path><path d="M10 24 Q24 13 38 24" stroke="{C["bedFurrow"]}" stroke-width="1.5" fill="none"></path>'
        if kind == "arbre":
            return '<circle cx="24" cy="18" r="13" fill="#5E9F5A"></circle><circle cx="20" cy="14" r="7" fill="#7DBE6E"></circle>'
        if kind == "haie":
            return '<rect x="3" y="13" width="42" height="14" rx="7" fill="#4E8C55"></rect><circle cx="12" cy="18" r="4" fill="#6BAA66"></circle><circle cx="26" cy="17" r="4" fill="#6BAA66"></circle><circle cx="38" cy="19" r="3.5" fill="#6BAA66"></circle>'
        if kind == "abri":
            return '<rect x="9" y="9" width="30" height="22" rx="2" fill="#B98A5E"></rect><path d="M9 9h30L24 20z" fill="#8E6440"></path>'
        if kind == "compost":
            return '<rect x="13" y="9" width="22" height="22" rx="3" fill="#7A5236"></rect><path d="M17 15h14M17 20h14M17 25h14" stroke="#5E3F28" stroke-width="2"></path>'
        if kind == "eau":
            return '<circle cx="24" cy="20" r="12" fill="#4F7FA6"></circle><circle cx="24" cy="20" r="8" fill="#7FB3D6"></circle>'
    tiles = [("planche", "Bed", "1.2 × 3 m", True), ("carre", "Square", "1 × 1 m", False), ("butte", "Mound", "1.2 × 3 m", False), ("bac", "Planter", "1.2 × 2 m", False)]
    palette = "".join(
        f'<button aria-label="Drag {t} onto the plot" style="flex: 1 1 88px; min-width: 80px; display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 10px 6px 8px; border-radius: 16px; cursor: grab; '
        f'border: {"2px dashed " + C["green"] if drag else "2px solid transparent"}; background: {"#E6F4EC" if drag else C["bg"]}; font: 600 12px Figtree, sans-serif; color: {C["ink"]}">'
        f'<svg width="48" height="40" viewBox="0 0 48 40" aria-hidden="true">{tile_svg(k)}</svg>{t}'
        f'{"<span style=" + chr(34) + "font-size: 11px; font-weight: 500; color: " + C["muted"] + chr(34) + ">" + sub + "</span>" if sub else ""}</button>'
        for k, t, sub, drag in tiles)
    tray = (f'<div style="display: flex; flex-direction: column; gap: 8px; padding: 12px; border-radius: 22px; background: #FFFFFF">'
            f'<div style="display: flex; justify-content: space-between; align-items: baseline; gap: 12px; padding: 0 4px"><strong style="font-size: 14px">Drag a bed onto the plot</strong>'
            f'<span style="font-size: 13px; color: {C["muted"]}">Beds 8 / 10</span></div>'
            f'<div style="display: flex; gap: 8px; overflow-x: auto">'
            f'{palette}</div></div>')
    field = lambda lid, lbl, val, unit="": (f'<label for="{lid}" style="flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; font-size: 12px; font-weight: 700; color: {C["muted"]}">{lbl}'
                                            f'<span style="display: flex; align-items: center; gap: 6px; padding: 0 12px; height: 44px; border-radius: 12px; background: {C["bg"]}">'
                                            f'<input id="{lid}" value="{val}" style="border: 0; background: transparent; font: 600 15px Figtree, sans-serif; color: {C["ink"]}; width: 100%; min-width: 0">'
                                            f'<span style="font-size: 13px; color: {C["muted"]}">{unit}</span></span></label>')
    types = "".join(f'<button aria-pressed="{"true" if on else "false"}" style="flex: 1; border: 0; padding: 8px 0; border-radius: 99px; font: 600 13px Figtree, sans-serif; cursor: pointer; '
                    f'{"background: " + C["ink"] + "; color: #FFFFFF" if on else "background: transparent; color: " + C["muted"]}">{t}</button>'
                    for t, on in (("Bed", True), ("Square", False), ("Mound", False), ("Planter", False)))
    panel = (
        f'<aside style="flex: 1 1 340px; max-width: 400px; min-width: 0; background: #FFFFFF; border-radius: 28px; padding: 24px; display: flex; flex-direction: column; gap: 18px">'
        f'<div style="display: flex; flex-direction: column; gap: 4px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Selected bed</span>'
        f'<h2 style="{H}; font-size: 24px">The tomatoes</h2></div>'
        f'{field("el-name", "Name", "The tomatoes")}'
        f'<div style="display: flex; flex-direction: column; gap: 6px"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">Type</span><div role="group" aria-label="Bed type" style="display: flex; padding: 4px; border-radius: 99px; background: {C["bg"]}">{types}</div></div>'
        f'<div style="display: flex; gap: 10px">{field("el-l", "Length", "4,0", "m")}{field("el-w", "Width", "1,0", "m")}</div>'
        f'<div style="display: flex; gap: 8px; flex-wrap: wrap">'
        + "".join(f'<button style="flex: 1 1 100px; display: inline-flex; justify-content: center; align-items: center; gap: 6px; border: 1px solid {C["line"]}; background: #FFFFFF; color: {col}; border-radius: 99px; padding: 11px 12px; font: 600 14px Figtree, sans-serif; cursor: pointer">{icon(ic, 16, col)}{t}</button>'
                  for ic, t, col in (("rotate", "Rotate", C["ink"]), ("copy", "Duplicate", C["ink"]), ("trash", "Delete", C["crit"])))
        + f'</div>'
        f'<p style="margin: 0; font-size: 13px; line-height: 1.5; color: {C["muted"]}">The crops and notes of the bed follow it when it is moved.</p>'
        f'<div style="display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; padding-top: 14px; border-top: 1px solid {C["line"]}; margin-top: auto">'
        + "".join(f'<div style="display: flex; flex-direction: column; gap: 2px"><strong style="{H}; font-size: 20px; font-variant-numeric: tabular-nums">{v}</strong><span style="font-size: 12px; color: {C["muted"]}">{t}</span></div>'
                  for v, t in (("71.5 m²", "Plot"), ("33.5 m²", "Workable"), ("8 / 10", "Beds")))
        + f'</div></aside>')
    header = (
        f'<header style="display: flex; flex-wrap: wrap; align-items: center; gap: 12px 18px; padding: 20px 32px">'
        f'<div style="display: flex; align-items: center; gap: 10px; margin-right: 8px">{LOGO}<span style="{H}; font-size: 26px; color: {C["ink"]}">Potagerie</span></div>'
        f'{mode_switch("Plan")}'
        f'<span style="display: inline-flex; align-items: center; gap: 8px; padding: 8px 14px; border-radius: 99px; background: #FFFFFF; border: 1px solid {C["line"]}; font-size: 14px"><strong>Plan 2027</strong><span style="color: {C["muted"]}">· applies to 2027 and the following seasons</span></span>'
        f'<div style="flex: 1 1 40px"></div>'
        f'<button aria-label="Undo the last action" style="width: 42px; height: 42px; border-radius: 50%; border: 1px solid {C["line"]}; background: #FFFFFF; display: grid; place-items: center; cursor: pointer">{icon("undo", 18)}</button>'
        f'<button aria-label="Redo" style="width: 42px; height: 42px; border-radius: 50%; border: 1px solid {C["line"]}; background: #FFFFFF; display: grid; place-items: center; cursor: pointer">{icon("redo", 18)}</button>'
        f'<a href="Main.dc.html" style="display: inline-flex; align-items: center; gap: 8px; height: 44px; padding: 0 20px; border-radius: 99px; background: {C["green"]}; color: #FFFFFF; text-decoration: none; font: 600 15px Figtree, sans-serif">{icon("check", 18, "#FFFFFF")}See the garden</a>'
        f'</header>')
    hints = "".join(f'<span style="display: inline-flex; align-items: center; gap: 8px; font-size: 13px; color: {C["muted"]}"><span style="display: inline-flex; align-items: center; gap: 6px; font: 600 12px Figtree, sans-serif; padding: 4px 9px; border-radius: 99px; background: #FFFFFF; border: 1px solid {C["line"]}; color: {C["ink"]}">{k}</span>{t}</span>'
                    for k, t in (("Tap", "a bed to select it"), ("Drag", "to move it (long press on a phone)"), ("Handles", "to resize it")))
    stage = (
        f'<section style="flex: 999 1 720px; min-width: 0; background: {C["card"]}; border-radius: 32px; padding: 20px; display: flex; flex-direction: column; gap: 14px">'
        f'<div style="display: flex; flex-wrap: wrap; gap: 8px 18px; align-items: center"><span style="flex: 1"></span>'
        f'<span style="font-size: 13px; color: {C["muted"]}">Terrain 11 × 6.5 m · 1 case = 50 cm</span></div>'
        f'<div style="position: relative">{plan_svg}'
        f'<div role="toolbar" aria-label="Actions on The tomatoes" style="position: absolute; left: 12.5%; top: 47%; display: flex; gap: 4px; padding: 6px; border-radius: 16px; background: {C["ink"]}; box-shadow: 0 8px 20px rgba(31,58,46,0.25)">'
        + "".join(f'<button aria-label="{t}" style="display: inline-flex; align-items: center; gap: 6px; height: 36px; padding: 0 12px; border-radius: 11px; border: 0; background: transparent; color: #FFFFFF; font: 600 13px Figtree, sans-serif; cursor: pointer">{icon(ic, 16, "#FFFFFF")}{t}</button>'
                  for ic, t in (("rotate", "Rotate"), ("copy", "Duplicate"), ("trash", "Delete")))
        + f'</div></div>'
        f'<div style="display: flex; flex-wrap: wrap; gap: 8px 16px">{hints}</div>'
        f'{tray}'
        f'</section>')
    body = (f'<div style="{ROOTSTYLE}; min-height: 980px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">{stage}{panel}</div></div>')
    return page("Plan editor — computer", body, {"$preview": {"width": 1440, "height": 980}})


BOARDS = {
    "Main.dc.html": main_board,
    "E05_garden_mobile.dc.html": mobile_board,
    "E07_planting_mobile.dc.html": serie_board,
    "DV01_visual-direction.dc.html": style_board,
    "E03_plan-editor_desktop.dc.html": atelier_board,
}

if __name__ == "__main__":
    for name, fn in BOARDS.items():
        with open(os.path.join(ROOT, name), "w", encoding="utf-8") as f:
            f.write(fn())
    canvas = {
        "v": 3,
        "createdOnFiles": {"v": 1, "at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")},
        "title": "POT-D01 · Key screens V1",
        "launch": {"view": "canvas"},
        "pages": [],
        "boards": {
            "E03_plan-editor_desktop.dc.html": {"x": 0, "y": 0, "w": 1440, "h": 980, "title": "① E03 · Plan editor — computer", "expand": "fill", "is_interactive": True},
            "Main.dc.html": {"x": 1640, "y": 0, "w": 1440, "h": 1560, "title": "② E05 · Garden — computer", "expand": "fill", "is_interactive": True},
            "E05_garden_mobile.dc.html": {"x": 0, "y": 1900, "w": 390, "h": 844, "title": "② E05 · Garden — phone", "is_interactive": True},
            "E07_planting_mobile.dc.html": {"x": 590, "y": 1900, "w": 390, "h": 1180, "title": "③ E07 · New planting — phone", "is_interactive": True},
            "DV01_visual-direction.dc.html": {"x": 1640, "y": 1900, "w": 1440, "h": 1080, "title": "DV01 · Visual direction"},
        },
        "order": ["Main.dc.html", "E03_plan-editor_desktop.dc.html", "E05_garden_mobile.dc.html", "E07_planting_mobile.dc.html", "DV01_visual-direction.dc.html"],
        "notes": {
            "titre": {"x": 0, "y": -300, "text": "Flow on a computer: design the plan, then plan the crops", "kind": "title1", "maxW": 3080},
            "lien1": {"x": 1460, "y": 40, "w": 160, "fill": "green", "size": "s",
                      "text": "Tabs at the top of the screen:\n“Garden” ↔ “Plan”.\n\n① → ② also with “See the garden”."},
            "lien2": {"x": 1460, "y": 360, "w": 160, "fill": "green", "size": "s",
                      "text": "② → ③\n“Plant here” or a free slot of the timeline opens the new planting."},
            "titre2": {"x": 0, "y": 1600, "text": "Flow on a phone", "kind": "title1", "maxW": 980},
            "lien3": {"x": 405, "y": 1940, "w": 170, "fill": "green", "size": "s",
                      "text": "② → ③\nThe “+” button of the bed.\n\n③ → ②\n“Plant” or the cross."},
            "titre3": {"x": 1640, "y": 1600, "text": "Visual direction", "kind": "title1", "maxW": 1440},
            "hyp": {"x": 3160, "y": 0, "w": 320, "fill": "orange",
                    "text": "Example: 2027 season, cursor on 20 June.\nExample garden of 7 beds (11 × 6.5 m), 10 at most, with the 6 reference crops (S02).\nThe Play button of each sheet follows the links.\nComing: crop sheet (E09) and export preview (E11)."},
        },
        "designSystems": [],
    }
    with open(os.path.join(ROOT, "canvas.json"), "w", encoding="utf-8") as f:
        json.dump(canvas, f, ensure_ascii=False, indent=1)
    print("ok", sorted(os.listdir(ROOT)))
