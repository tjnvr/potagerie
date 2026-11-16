"""Crop drawings of the diorama: faceted low-poly, light from the left.

Builds on the helpers of POT-D01 plants2.py (leaf, lobed, trifoliate, mound,
flower5) and draws each crop at 4 stages: 0 sowing, 1 young shoot, 2 adult,
3 harvest. Every plant is centred on (0, 0), its foot on the soil. Three tones
per surface (light left facet, dark right facet, highlight) and a two-layer
contact shadow; no filter, no gradient, a few dozen shapes at most.
"""
import math
import os
import re
import sys

D01 = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                   "docs", "design", "POT-D01_key-screens_v1.2")
sys.path.insert(0, D01)
from plants2 import (_circle, _ell, _line, _path, _poly, flower5, leaf, lobed,  # noqa: E402
                     mound, trifoliate)

VEIN = "#3F7F3C"
SOIL = "#8E5F3D"
CROPS = ["tomato", "lettuce", "carrot", "green-bean", "zucchini", "leek",
         "pumpkin", "cabbage", "broccoli", "onion", "beetroot", "sweet-corn", "bell-pepper", "eggplant"]
# Height of each plant at its tallest (units above the foot), for the icon framing.
HEIGHT = {"tomato": 55, "lettuce": 20, "carrot": 24, "green-bean": 52, "zucchini": 28, "leek": 32,
          "pumpkin": 28, "cabbage": 22, "broccoli": 28, "onion": 30, "beetroot": 28,
          "sweet-corn": 58, "bell-pepper": 36, "eggplant": 38}


def one(v):
    return v


def shade(cx, cy, rx):
    """Soft contact shadow: a wide faint ellipse and a darker core."""
    return (_ell(cx + 1, cy + 1.5, rx, rx * 0.42, "#2A4A30", 0, ' fill-opacity="0.16"')
            + _ell(cx + 0.5, cy + 1, rx * 0.6, rx * 0.26, "#2A4A30", 0, ' fill-opacity="0.2"'))


def frill(cx, cy, rx, ry, light, dark, n=12, amp=0.14, rot=0):
    """Ruffled leaf blob: scalloped ellipse, left half light, right half dark."""
    pts = []
    for i in range(n):
        a = math.radians(90 + i * 360 / n + rot)
        r = 1 + (amp if i % 2 else -amp * 0.4)
        pts.append((cx + math.cos(a) * rx * r, cy + math.sin(a) * ry * r))
    h = n // 2
    return _poly(pts[: h + 1], light) + _poly(pts[h:] + [pts[0]], dark)


def fruit(x, y, r, col, dark="#000000"):
    """Round fruit with a shaded side, a highlight and a small calyx."""
    s = _circle(x, y, r, col) + _ell(x + r * 0.28, y + r * 0.2, r * 0.7, r * 0.72, dark, 0, ' fill-opacity="0.16"')
    s += _circle(x - r * 0.4, y - r * 0.4, r * 0.28, "#FFFFFF", ' fill-opacity="0.5"')
    return s + _poly([(x - r * 0.5, y - r * 0.8), (x, y - r * 1.05), (x + r * 0.5, y - r * 0.8), (x, y - r * 0.45)], "#3E7E3A")


def blade(x, y, ang, L, W, light, dark, bend=0.0):
    """Long thin blade with a slight bend: two facets split by the midrib."""
    a = math.radians(ang)
    dx, dy = math.sin(a), -math.cos(a)
    nx, ny = math.cos(a), math.sin(a)
    tx, ty = x + dx * L + nx * bend * L, y + dy * L + ny * bend * L
    mx, my = x + dx * L * 0.5 + nx * bend * L * 0.5, y + dy * L * 0.5 + ny * bend * L * 0.5
    return (_poly([(x, y), (mx - nx * W / 2, my - ny * W / 2), (tx, ty)], light)
            + _poly([(x, y), (mx + nx * W / 2, my + ny * W / 2), (tx, ty)], dark))


def sprout(kind):
    s = mound(0, 0, one)
    if kind in ("leek", "onion", "sweet-corn"):
        return s + _path("M0 0 q-1 -4 -2.5 -7", "#7DB56B", 1.3) + _path("M1 0 q1 -3 2 -5.5", "#6AA65C", 1.3)
    return (s + _line(0, 0, 0, -4, "#4C8F48", 1) + _ell(-2.4, -4.6, 2.6, 1.3, "#86CC70", -20)
            + _ell(2.4, -4.8, 2.6, 1.3, "#62AE56", 20))


def young(kind):
    s = shade(0, 0, 7) + mound(0, 0, one)
    G1, G2 = "#7CC46A", "#5FAA55"
    if kind in ("lettuce", "cabbage", "broccoli", "beetroot", "carrot"):
        cols = {"cabbage": ("#8FC0A4", "#6FA188"), "broccoli": ("#5E9C6C", "#437F55"),
                "beetroot": ("#6DB24D", "#4F9A3F"), "carrot": ("#9AD46E", "#78BC57")}.get(kind, ("#9AD46E", "#74B04E"))
        for a, L in ((-62, 7), (58, 7), (-30, 9), (28, 9), (0, 8)):
            s += leaf(0, -1, a, L, L * 0.55, cols[0], cols[1])
        if kind == "beetroot":
            s += _line(0, -1, 0, -7, "#B0304F", 1.1)
    elif kind in ("leek", "onion", "sweet-corn"):
        n = ((-2.5, 12), (0, 15), (2.5, 11)) if kind != "sweet-corn" else ((-4, 12), (0, 17), (4, 13))
        for dx, h in n:
            s += _path(f"M0 0 Q{dx * 0.4:.1f} {-h * 0.6:.1f} {dx:.1f} {-h:.1f}", "#6FB59A" if kind == "leek" else "#7DB56B", 1.7)
    elif kind in ("zucchini", "pumpkin"):
        s += _line(0, 0, 0, -5, "#4C8F48", 1.2) + _ell(-3.5, -5.5, 3.6, 2, "#8ACB6C", -15) + _ell(3.5, -5.8, 3.6, 2, "#67AE58", 15)
        s += lobed(1, -9, 4.8, "#79BE62", "#5C9F4F", VEIN, one)
    elif kind == "green-bean":
        s += _line(0, 0, 0, -10, "#4C8F48", 1.2) + trifoliate(0, -8, -1, one, "#86CC70", "#62AE56", VEIN, 4.2)
        s += trifoliate(0, -10, 1, one, "#86CC70", "#62AE56", VEIN, 4)
    else:  # tomato, bell-pepper, eggplant
        s += _line(0, 0, 0, -8, "#4C8F48", 1.1)
        for a, h in ((-55, 3), (50, 4), (-25, 6.5), (30, 7)):
            s += leaf(0, -h, a, 5.5, 3.6, G1, G2)
    return s


def tomato(stage):
    harvest = stage == 3
    s = shade(0, 0, 13) + _line(6, 1, 6, -54, "#9C6B3E", 1.8)
    s += _path("M0 0 Q-3 -22 1 -46", "#4E8F45", 2)
    for h, a, L in ((7, -62, 14), (11, 58, 14), (18, -52, 15), (22, 62, 14), (29, -48, 13), (33, 55, 12), (40, -35, 10), (44, 30, 9)):
        bx, by = -2 + h * 0.07, -h
        s += leaf(bx, by, a, L, L * 0.5, "#6CBB62", "#4C944D", VEIN, 0.6)
        ex, ey = bx + math.sin(math.radians(a)) * L * 0.5, by - math.cos(math.radians(a)) * L * 0.5
        s += leaf(ex, ey, a + (28 if a > 0 else -28), L * 0.42, L * 0.28, "#7CC66E", "#589F55")
    s += _line(6, -16, 1, -16, "#E6D3A8", 1.1) + _line(6, -34, 1, -34, "#E6D3A8", 1.1)
    cols = ("#E5533D", "#EA6342", "#D9402F") if harvest else ("#9CCB6A", "#B4D66F", "#E9A35A")
    for (tx, ty), c in zip(((-9, -13), (10, -24), (-7, -34)), cols):
        r = 3.3 if harvest else 2.3
        s += _path(f"M{tx + 3} {ty - 6} Q{tx + 3} {ty - 3} {tx} {ty - 2}", "#4E8F45", 0.8)
        s += fruit(tx - 2, ty + 1, r, c) + fruit(tx + 2.6, ty + 2.2, r * 0.9, cols[0] if harvest else "#A9D06C")
        if harvest:
            s += fruit(tx, ty + 5.5, r * 0.85, c)
    if not harvest:
        for fx, fy in ((-4, -40), (8, -30)):
            s += flower5(fx, fy, 1.5, "#F6C443", "#E08A1E", one)
    return s


def lettuce(stage):
    big = stage == 3
    s = shade(0, 0, 12 if big else 10)
    for dx, rx, ry, l, d in ((-8, 9, 7.5, "#5E9E48", "#4A8A3C"), (8, 9, 7.5, "#68AA50", "#4F8F40"), (0, 10, 7, "#76B656", "#5A9A47")):
        s += frill(dx * (1.15 if big else 1), -4 if dx else -2.5, rx * (1.1 if big else 1), ry, l, d, 12, 0.16)
    for dx, y, rx, l, d in ((-5, -6, 6.5, "#8CC862", "#6FAE4E"), (5, -6, 6.5, "#92CC66", "#72B151"), (0, -8, 6.5, "#9CD470", "#7DBA58")):
        s += frill(dx, y - (1 if big else 0) - 1, rx, 6.5, l, d, 10, 0.15)
    hr = 6.2 if big else 4.4
    s += frill(0, -12 - (2 if big else 0), hr, hr * 0.8, "#C6E68E", "#A8CF72", 8, 0.12)
    s += _path(f"M-2 {-9 - (2 if big else 0)} q2 -1.5 4 0", "#8FBF5E", 0.5)
    return s


def carrot(stage):
    harvest = stage == 3
    s = shade(0, 0, 9)
    for a, L in ((-40, 16), (-22, 21), (-4, 24), (14, 21), (32, 17)):
        ra = math.radians(a)
        s += _line(0, -1, math.sin(ra) * L, -1 - math.cos(ra) * L, "#5FA445", 0.9)
        for t in (0.5, 0.78):
            px, py = math.sin(ra) * L * t, -1 - math.cos(ra) * L * t
            sz = 5.4 * (1.15 - t * 0.5)
            s += leaf(px, py, a - 50, sz, sz * 0.5, "#9AD46E", "#7CBF5A") + leaf(px, py, a + 50, sz, sz * 0.5, "#8ACB62", "#6CB04D")
        ex, ey = math.sin(ra) * L, -1 - math.cos(ra) * L
        s += leaf(ex, ey, a - 18, 4, 2, "#A6DA78", "#82C25E") + leaf(ex, ey, a + 18, 4, 2, "#92CE68", "#72B652")
    # visible orange shoulder at the soil line
    s += _ell(0, -0.5, 4.6, 2.4, SOIL) + _ell(-0.4, -1.4, 3.6, 1.7, "#EF8A2E") + _ell(1.2, -1.2, 2, 1.2, "#D9731F") + _ell(-1.6, -2, 1.2, 0.6, "#FFC27A")
    s += _ell(0, -2.4, 1.8, 0.8, "#7DB05A")
    if harvest:
        s += _poly([(-2, 8), (-1, 4), (3, 3), (8, 4), (18, 9), (8, 7.5)], "#EF8A2E") + _poly([(-1, 5), (3, 3), (8, 4), (18, 9), (7, 8)], "#D9731F")
        s += _line(4, 4.5, 4.4, 7, "#C9661A", 0.5) + _line(8, 5.2, 8.2, 7.4, "#C9661A", 0.5) + _line(12, 6.4, 12.2, 8, "#C9661A", 0.5)
        s += _line(-3, 7, -6, 5, "#5FA445", 1.2) + _line(-3, 7, -6, 9, "#6DB24D", 1.2)
    return s


def green_bean(stage):
    harvest = stage == 3
    s = shade(0, 0, 10)
    s += _line(-7, 1, 3, -52, "#A07448", 1.6) + _line(7, 1, -3, -52, "#9A6E42", 1.6)
    s += _path("M0 0 " + " ".join(f"Q{4 if i % 2 == 0 else -4} {-i * 9 - 4.5} 0 {-i * 9 - 9}" for i in range(5)), "#5FA445", 1.1)
    for i, h in enumerate((8, 15, 22, 29, 36, 43)):
        side = -1 if i % 2 == 0 else 1
        s += trifoliate(side * 1.5, -h, side, one, "#86CC70", "#5FA852", VEIN, 6.2 - h * 0.03)
    if harvest:
        for x0, y0, c in ((-5, 14, "#8CC63F"), (5, 21, "#9BCB45"), (-4.5, 28, "#8CC63F"), (4.5, 35, "#7FB93A"), (-3, 40, "#9BCB45")):
            s += _path(f"M{x0 * 0.6} {-y0} q{x0 * 0.3:.1f} 5 {x0 * 0.15:.1f} 11", c, 2.1) + _path(f"M{x0 * 0.6 - 0.5:.1f} {-y0 + 1} q{x0 * 0.3:.1f} 4 {x0 * 0.15:.1f} 8", "#C4E27A", 0.5)
    else:
        for x0, y0 in ((-6, 16), (6, 24), (-5, 33), (5, 40)):
            s += _circle(x0, -y0, 1.5, "#F4EEF8") + _circle(x0 + 0.8, -y0 - 0.5, 0.9, "#D9C9EE")
    return s


def zucchini(stage):
    harvest = stage == 3
    s = shade(0, 0, 19)
    for x, y, r, rot, l, d in ((-10, -13, 9.5, 8, "#6FB25C", "#518F47"), (11, -16, 10.5, -12, "#7ABD66", "#5A9A4E"), (0, -22, 9, 20, "#82C46C", "#5F9F51")):
        s += _line(0, -2, x * 0.7, y + 4, "#5C9A4C", 1.3) + lobed(x, y, r, l, d, VEIN, one, rot)
    for fx, fy, ang, ln in (((-5, 4, -14, 11), (6, 3, 10, 9)) if harvest else ((-5, 4, -14, 7),)):
        s += _ell(fx, fy, ln, 3.3, "#3A8344", ang) + _ell(fx + 0.5, fy + 1.1, ln * 0.85, 1.5, "#2B6A38", ang, ' fill-opacity="0.7"')
        s += _ell(fx - 1, fy - 1.2, ln * 0.6, 0.8, "#8DD07A", ang, ' fill-opacity="0.7"')
        s += _circle(fx - ln * math.cos(math.radians(ang)) + 0.5, fy - ln * math.sin(math.radians(ang)), 0.9, "#7A8A3A")
    if not harvest:
        s += flower5(10, -4, 2.8, "#F6C443", "#E08A1E", one) + _line(10, -2, 7, 3, "#4E8F45", 1.2)
    return s


def leek(stage):
    harvest = stage == 3
    h = 17 if harvest else 11
    w = 2.6 if harvest else 2.1
    s = shade(0, 0, 7)
    s += _poly([(-w, 0), (w, 0), (w * 0.85, -h), (-w * 0.85, -h)], "#F4F7E8") + _poly([(0.2, 0), (w, 0), (w * 0.85, -h), (0.2, -h)], "#D9E4C4")
    s += _poly([(-w * 0.85, -h), (w * 0.85, -h), (0, -h - 3.5)], "#CFE3C8") + _line(-w, -1, w, -1, "#BFD0A6", 0.5)
    for i, (a, L, c) in enumerate(((-42, 19, 0), (38, 20, 1), (-24, 25, 1), (22, 26, 0), (-8, 25, 0), (8, 28, 1))):
        l, d = (("#7BC0A4", "#55A088") if c == 0 else ("#68B09A", "#4A9080"))
        s += blade(0, -h + 1 + i * 0.5, a, L * (1.1 if harvest else 0.9), 5, l, d, 0.06 * (1 if a > 0 else -1))
    s += _line(-1, 1, -3, 3, "#EEE6D2", 0.6) + _line(0.5, 1, 1, 3.5, "#EEE6D2", 0.6) + _line(1.5, 1, 3.5, 2.6, "#EEE6D2", 0.6)
    return s


def pumpkin(stage):
    harvest = stage == 3
    s = shade(0, 0, 19)
    for x, y, r, rot, l, d in ((-11, -12, 9, 8, "#5F9F54", "#488540"), (12, -15, 10, -12, "#69AA5B", "#4F8F47"), (2, -22, 8.5, 20, "#72B263", "#559549")):
        s += _line(0, -2, x * 0.7, y + 4, "#4F8F45", 1.3) + lobed(x, y, r, l, d, VEIN, one, rot)
    s += _path("M-18 2 Q-6 -3 -2 0", "#5C9A4C", 1.2)
    if harvest:
        for fx, fy, rx, ry, c1, c2 in ((-1, 2, 9.5, 7.5, "#EE8A33", "#D96A1C"), (11, 4, 5, 4, "#F09A45", "#DC7424")):
            s += _ell(fx, fy, rx, ry, c2) + _ell(fx - rx * 0.25, fy, rx * 0.7, ry, c1) + _ell(fx - rx * 0.25, fy, rx * 0.28, ry, c2, 0, ' fill-opacity="0.55"')
            s += _ell(fx + rx * 0.5, fy, rx * 0.3, ry * 0.95, "#B85612", 0, ' fill-opacity="0.35"')
            s += _circle(fx - rx * 0.5, fy - ry * 0.45, rx * 0.14, "#FFFFFF", ' fill-opacity="0.45"') + _poly([(fx - 1, fy - ry), (fx + 1, fy - ry), (fx + 1.6, fy - ry - 3), (fx - 0.8, fy - ry - 3.2)], "#7A6A2A")
    else:
        s += _ell(5, 3, 4, 3.2, "#5C9A4C") + _ell(4, 3, 2.4, 3.2, "#7DBA62") + flower5(-8, -4, 2.8, "#F6C443", "#E08A1E", one)
    return s


def cabbage(stage):
    harvest = stage == 3
    s = shade(0, 0, 14)
    for dx, rx, l, d in ((-9, 10, "#6FA088", "#578570"), (9, 10, "#7AAA92", "#5F8E78")):
        s += frill(dx, -5, rx, 6.5, l, d, 10, 0.1)
    s += frill(0, -8, 10, 7.5, "#86B79C", "#6A9C84", 10, 0.1)
    r = 8.5 if harvest else 5.5
    s += _ell(0, -6 - r * 0.6, r, r * 0.95, "#A9D2A0") + _ell(r * 0.35, -6 - r * 0.55, r * 0.7, r * 0.9, "#8DBB88", 0, ' fill-opacity="0.8"')
    s += _path(f"M{-r * 0.2:.1f} {-6 - r * 1.5:.1f} Q{-r * 0.9:.1f} {-6 - r * 0.6:.1f} {-r * 0.3:.1f} -5", "#7DAF7A", 0.6) + _path(f"M{r * 0.35:.1f} {-6 - r * 1.5:.1f} Q{r * 0.95:.1f} {-6 - r * 0.6:.1f} {r * 0.4:.1f} -5", "#7DAF7A", 0.6)
    s += _ell(-r * 0.35, -6 - r * 1.05, r * 0.22, r * 0.32, "#E8F4DC", 0, ' fill-opacity="0.6"')
    return s


def broccoli(stage):
    harvest = stage == 3
    s = shade(0, 0, 13)
    for a, L in ((-72, 17), (66, 17), (-44, 21), (40, 21), (-18, 20), (14, 19)):
        s += leaf(0, -2, a, L, L * 0.42, "#5E9E78", "#437F5F", "#35694D", 0.5)
    sh = 11 if harvest else 6
    s += _poly([(-1.8, -2), (1.8, -2), (1.4, -sh), (-1.4, -sh)], "#8DBF7A") + _poly([(0.1, -2), (1.8, -2), (1.4, -sh), (0.1, -sh)], "#6FA862")
    r = 8.5 if harvest else 4.5
    cy = -sh - r * 0.35
    for dx, dy, k in ((-0.6, 0.2, 0.62), (0.55, 0.15, 0.6), (0, -0.55, 0.58), (-0.15, 0.55, 0.5)):
        s += _circle(dx * r, cy + dy * r, r * k, "#3F7D4E")
    for dx, dy, k in ((-0.6, 0.05, 0.4), (0.0, -0.7, 0.38), (0.55, 0.0, 0.3)):
        s += _circle(dx * r - r * 0.12, cy + dy * r - r * 0.1, r * k, "#5BA064")
    s += _circle(-r * 0.5, cy - r * 0.35, r * 0.15, "#9ED18E")
    return s


def onion(stage):
    harvest = stage == 3
    s = shade(0, 0, 9)
    cols = ("#B9B56A", "#C8C37A", "#A9A55A") if harvest else ("#7DB56B", "#8BC27A", "#6AA65C", "#79B168")
    spots = ((-7, 9), (0, 13), (6, 8)) if harvest else ((-5, 21), (-1.5, 26), (2.5, 23), (6, 17))
    for (dx, h), c in zip(spots, cols):
        if harvest:  # leaves folded over
            s += _path(f"M{dx * 0.2:.1f} -6 Q{dx * 0.6:.1f} {-h - 3} {dx * 1.6:.1f} {-h + 1}", c, 1.9)
        else:
            s += _path(f"M0 -5 Q{dx * 0.3:.1f} {-h * 0.7:.1f} {dx} {-h}", c, 2.1)
    br = 6.2 if harvest else 4.6
    by = -br * 0.7 + 1.5
    s += _ell(0, by, br, br * 0.85, "#E6B26A") + _ell(br * 0.35, by + 0.2, br * 0.62, br * 0.8, "#C98F4E", 0, ' fill-opacity="0.65"')
    s += _ell(-br * 0.4, by - br * 0.3, br * 0.22, br * 0.3, "#FFE2A8", 0, ' fill-opacity="0.7"')
    s += _path(f"M{-br * 0.35:.1f} {by - br * 0.8:.1f} Q{-br * 0.7:.1f} {by:.1f} {-br * 0.3:.1f} {by + br * 0.8:.1f}", "#B97D3E", 0.5)
    s += _path(f"M{br * 0.3:.1f} {by - br * 0.8:.1f} Q{br * 0.75:.1f} {by:.1f} {br * 0.35:.1f} {by + br * 0.8:.1f}", "#B97D3E", 0.5)
    s += _poly([(-1, by - br * 0.8), (1, by - br * 0.8), (0.4, by - br * 1.2), (-0.4, by - br * 1.2)], "#C98F4E")
    if harvest:
        s += _line(-1, by + br * 0.8, -2, by + br, "#F0E4C8", 0.5) + _line(0.5, by + br * 0.8, 1, by + br, "#F0E4C8", 0.5)
    else:
        s += _ell(0, 1.8, br + 1.4, 1.7, SOIL)
    return s


def beetroot(stage):
    harvest = stage == 3
    s = shade(0, 0, 11)
    for a, L, pet in ((-52, 19, 8), (50, 19, 8), (-28, 24, 10), (26, 24, 10), (0, 26, 11)):
        ra = math.radians(a)
        px, py = math.sin(ra) * pet, -2 - math.cos(ra) * pet
        s += _line(0, -2, px, py, "#B0304F", 1.4)
        s += leaf(px, py, a, L - pet * 0.4, (L - pet) * 0.75, "#5FA450", "#437F3F", "#B0304F", 0.6)
    if harvest:
        s += _poly([(-1, 5), (1, 5), (0.2, 12)], "#6E1F38") + _circle(-1, 0, 6.8, "#B02E58") + _ell(1.8, 0.8, 4.8, 6, "#7E2142", 0, ' fill-opacity="0.65"')
        s += _circle(-3.4, -3, 1.6, "#FFFFFF", ' fill-opacity="0.45"') + _path("M-5 1 Q-1 3 4 1", "#8E2A4A", 0.4)
    else:
        s += _ell(0, -0.5, 5.2, 2.8, SOIL) + _ell(-0.4, -1.8, 4.2, 2.4, "#B02E58") + _ell(1.4, -1.5, 2.4, 1.8, "#7E2142") + _ell(-1.8, -2.6, 1.2, 0.6, "#E58AA4")
    return s


def sweet_corn(stage):
    harvest = stage == 3
    s = shade(0, 0, 11)
    s += _poly([(-2.4, 1), (2.4, 1), (1.7, -50), (-1.7, -50)], "#8EC465") + _poly([(0, 1), (2.4, 1), (1.7, -50), (0, -50)], "#6FAA4F")
    for k in range(1, 6):
        s += _line(-2.2 + k * 0.1, 1 - k * 9, 2.2 - k * 0.1, 1 - k * 9, "#5E9A45", 0.6)
    for h, a, L in ((6, -78, 24), (11, 74, 25), (18, -66, 26), (25, 70, 25), (32, -54, 22), (38, 58, 20), (44, -34, 15), (46, 30, 14)):
        s += blade(0, -h, a, L, 5.2, "#78BC58", "#589E43", -0.18 if a < 0 else 0.18)
    if harvest:
        for ex, ey, side in ((4, -28, 1), (-4, -20, -1)):
            ang = side * 12
            s += _ell(ex + side * 3, ey - 7, 3.4, 9, "#6FAE4C", ang) + _ell(ex + side * 4, ey - 7, 1.8, 8.4, "#4F8F3A", ang, ' fill-opacity="0.6"')
            s += _ell(ex + side * 4.2, ey - 17, 2.6, 4, "#F2C233", ang) + _ell(ex + side * 3.6, ey - 17.5, 1.1, 3.4, "#FFE07A", ang)
            s += _line(ex + side * 4.2, ey - 20, ex + side * 5.5, ey - 23, "#B98A4A", 0.9)
    else:
        for x2, y2, c, w in ((0, -58, "#D9C26A", 0.9), (-4, -56, "#D9C26A", 0.9), (4, -56, "#D9C26A", 0.9), (-2, -57, "#C9B05A", 0.7), (2, -57, "#C9B05A", 0.7)):
            s += _line(0, -50, x2, y2, c, w)
        s += _ell(7, -34, 3, 8, "#6FAE4C", 10) + _ell(8, -34, 1.6, 7.4, "#4F8F3A", 10, ' fill-opacity="0.6"') + _line(8.5, -42, 10, -46, "#B98A4A", 0.9)
    return s


def bell_pepper(stage):
    harvest = stage == 3
    s = shade(0, 0, 12) + _path("M0 0 Q-2 -14 1 -28", "#4E8F45", 2)
    for h, a, L in ((6, -60, 11), (9, 56, 11), (15, -50, 12), (19, 58, 11), (25, -40, 10), (28, 35, 9), (31, -10, 6)):
        s += leaf(-0.5, -h, a, L, L * 0.52, "#6CBB62", "#4C944D", VEIN, 0.5)
    c1, c2 = ("#E04030", "#B22A1E") if harvest else ("#7DBD4E", "#5C9A38")
    sc = 1.4 if harvest else 0.9
    for fx, fy in ((-8, -10), (8, -16)):
        s += _line(fx * 0.3, fy - 7 * sc, fx * 0.3 + 0.5, fy - 12, "#4E8F45", 1)
        pts = [(-3.6, -4.5), (3.6, -4.5), (4.6, 0.5), (3.2, 4.8), (0, 3.6), (-3.2, 4.8), (-4.6, 0.5)]
        s += _poly([(fx + x * sc, fy + y * sc) for x, y in pts], c1)
        s += _poly([(fx + x * sc, fy + y * sc) for x, y in [(0.4, -4.5), (3.6, -4.5), (4.6, 0.5), (3.2, 4.8), (0.4, 3.9)]], c2)
        s += _ell(fx - 2 * sc, fy - 1.5 * sc, 0.8 * sc, 2 * sc, "#FFFFFF", 0, ' fill-opacity="0.45"')
        s += _poly([(fx - 1.8 * sc, fy - 4.6 * sc), (fx + 1.8 * sc, fy - 4.6 * sc), (fx, fy - 6.2 * sc)], "#3E7E3A")
    if not harvest:
        s += flower5(-3, -30, 1.4, "#FFFFFF", "#F2C232", one)
    return s


def eggplant(stage):
    harvest = stage == 3
    s = shade(0, 0, 13) + _path("M0 0 Q-2 -16 1 -30", "#6B8F4A", 2.2)
    for h, a, L in ((6, -62, 14), (10, 60, 14), (17, -52, 15), (21, 56, 14), (27, -42, 12), (30, 38, 11), (33, -8, 8)):
        s += leaf(-0.5, -h, a, L, L * 0.62, "#6FA06A", "#527F52", "#6B3A7A", 0.55)
    if harvest:
        for fx, fy, ang in ((-7, -9, 14), (8, -15, -12)):
            s += _line(fx, fy - 7, fx * 0.3, fy - 15, "#6B8F4A", 1.1)
            s += _ell(fx, fy, 4.2, 8.5, "#4A2060", ang) + _ell(fx - 1, fy, 2.8, 7.4, "#6A3480", ang, ' fill-opacity="0.9"') + _ell(fx - 1.8, fy - 2, 0.8, 3.2, "#FFFFFF", ang, ' fill-opacity="0.45"')
            s += _poly([(fx - 3.2, fy - 7), (fx + 3.2, fy - 7), (fx + 1.2, fy - 4.8), (fx - 1.2, fy - 4.8)], "#4E8F45") + _poly([(fx - 2, fy - 8.4), (fx + 2, fy - 8.4), (fx, fy - 7)], "#3E7E3A")
    else:
        for fx, fy in ((-4, -26), (7, -20)):
            s += flower5(fx, fy, 1.9, "#B48ACE", "#F2C232", one)
    return s


DRAW = {"tomato": tomato, "lettuce": lettuce, "carrot": carrot, "green-bean": green_bean, "zucchini": zucchini,
        "leek": leek, "pumpkin": pumpkin, "cabbage": cabbage, "broccoli": broccoli, "onion": onion,
        "beetroot": beetroot, "sweet-corn": sweet_corn, "bell-pepper": bell_pepper, "eggplant": eggplant}


def compact(svg):
    """Shorter markup: shared round caps in a group, self-closing shapes."""
    svg = svg.replace(' stroke-linecap="round"', "").replace(' stroke-linejoin="round"', "")
    svg = re.sub(r"></(polygon|line|circle|ellipse|path)>", "/>", svg)
    svg = re.sub(r"(\d)\.0\b", r"\1", svg)
    return f'<g stroke-linecap="round" stroke-linejoin="round">{svg}</g>'


def plant(crop, stage):
    if stage == 0:
        return compact(sprout(crop))
    if stage == 1:
        return compact(young(crop))
    return compact(DRAW[crop](stage))
