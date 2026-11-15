"""Detailed crops, faceted low-poly style (light coming from the left)."""
import math


def _f(v):
    return f"{v:.1f}"


def _poly(ps, fill, extra=""):
    return f'<polygon points="{" ".join(_f(x) + "," + _f(y) for x, y in ps)}" fill="{fill}"{extra}></polygon>'


def _line(x1, y1, x2, y2, col, w, extra=""):
    return f'<line x1="{_f(x1)}" y1="{_f(y1)}" x2="{_f(x2)}" y2="{_f(y2)}" stroke="{col}" stroke-width="{_f(w)}" stroke-linecap="round"{extra}></line>'


def _circle(x, y, r, col, extra=""):
    return f'<circle cx="{_f(x)}" cy="{_f(y)}" r="{_f(r)}" fill="{col}"{extra}></circle>'


def _ell(x, y, rx, ry, col, ang=0, extra=""):
    t = f' transform="rotate({ang:.1f} {_f(x)} {_f(y)})"' if ang else ""
    return f'<ellipse cx="{_f(x)}" cy="{_f(y)}" rx="{_f(rx)}" ry="{_f(ry)}" fill="{col}"{t}{extra}></ellipse>'


def _path(d, col, w, fill="none"):
    return f'<path d="{d}" stroke="{col}" stroke-width="{_f(w)}" stroke-linecap="round" stroke-linejoin="round" fill="{fill}"></path>'


def leaf(x, y, ang, L, W, light, dark, vein=None, vw=0.6):
    """Diamond leaf in two facets. ang in degrees from the vertical (positive = to the right)."""
    a = math.radians(ang)
    dx, dy = math.sin(a), -math.cos(a)
    nx, ny = math.cos(a), math.sin(a)
    tip = (x + dx * L, y + dy * L)
    mx, my = x + dx * L * 0.42, y + dy * L * 0.42
    lp = (mx - nx * W / 2, my - ny * W / 2)
    rp = (mx + nx * W / 2, my + ny * W / 2)
    s = _poly([(x, y), lp, tip], light) + _poly([(x, y), rp, tip], dark)
    if vein:
        s += _line(x, y, x + dx * L * 0.85, y + dy * L * 0.85, vein, vw)
    return s


def shadow(cx, cy, rx, k):
    return _ell(cx, cy + 1, rx * k, rx * 0.42 * k, "#2A4A30", 0, ' fill-opacity="0.18"')


def mound(cx, cy, g):
    return _ell(cx, cy, g(6), g(2.4), "#8E5F3D")


def calyx(x, y, g, col="#3E7E3A"):
    s = ""
    for a in (-70, -20, 25, 70, 160):
        r = math.radians(a)
        s += _line(x, y, x + math.cos(r) * g(2.2), y + math.sin(r) * g(1.3) - g(0.6), col, g(0.8))
    return s


def tomato(x, y, r, col, g):
    return (_circle(x, y, r, col) + _ell(x + r * 0.25, y + r * 0.2, r * 0.75, r * 0.7, "#000000", 0, ' fill-opacity="0.10"')
            + _circle(x - r * 0.38, y - r * 0.38, r * 0.3, "#FFFFFF", ' fill-opacity="0.55"') + calyx(x, y - r * 0.85, g))


def lobed(cx, cy, r, light, dark, vein, g, rot=0):
    """Large lobed leaf (cucurbit), flattened in perspective."""
    pts = []
    for i in range(10):
        a = math.radians(rot + i * 36)
        rr = r if i % 2 == 0 else r * 0.68
        pts.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr * 0.62))
    left = [p for p in pts if p[0] <= cx] or pts
    s = _poly(pts, dark)
    s += _poly([(cx, cy - r * 0.62)] + sorted(left, key=lambda p: math.atan2(p[1] - cy, p[0] - cx)) + [(cx, cy + r * 0.62)], light)
    for a in (rot - 90, rot - 18, rot + 54, rot + 126, rot + 198):
        ra = math.radians(a)
        s += _line(cx, cy, cx + math.cos(ra) * r * 0.75, cy + math.sin(ra) * r * 0.75 * 0.62, vein, g(0.7))
    return s


def trifoliate(x, y, side, g, light, dark, vein, size=5.0):
    s = ""
    for da in (-42, 0, 42):
        s += leaf(x, y, side * 55 + da, g(size), g(size * 0.62), light, dark, vein, g(0.45))
    return s


def flower5(x, y, r, petal, center, g):
    s = ""
    for i in range(5):
        a = math.radians(i * 72 - 90)
        s += _circle(x + math.cos(a) * r, y + math.sin(a) * r * 0.8, r * 0.75, petal)
    return s + _circle(x, y, r * 0.55, center)


def strawberry(x, y, g):
    s = _poly([(x - g(2.6), y - g(1.2)), (x + g(2.6), y - g(1.2)), (x + g(1.6), y + g(2)), (x, y + g(3.6)), (x - g(1.6), y + g(2))], "#E0384A")
    s += _poly([(x, y - g(1.2)), (x + g(2.6), y - g(1.2)), (x + g(1.6), y + g(2)), (x, y + g(3.6))], "#C42A3C")
    for dx, dy in ((-1.1, 0.2), (0.9, 0.4), (-0.3, 1.6), (1.2, 1.9), (-1.2, 2.0)):
        s += _circle(x + g(dx), y + g(dy), g(0.32), "#F6D86B")
    s += _poly([(x - g(2.4), y - g(1.3)), (x, y - g(2.6)), (x + g(2.4), y - g(1.3)), (x, y - g(0.6))], "#3F8A43")
    return s


def plant(kind, stage, cx, cy, k=1.0):
    """stage: 0 sowing, 1 young shoot, 2 adult, 3 harvest."""
    g = lambda v: v * k
    G1, G2, VEIN = "#7CC46A", "#5FAA55", "#3F7F3C"
    s = ""
    # ---- semis ----
    if stage == 0:
        s += mound(cx, cy, g)
        if kind in ("poireau", "oignon"):
            s += _path(f"M{_f(cx)} {_f(cy)} q {_f(g(-1))} {_f(g(-4))} {_f(g(-2.5))} {_f(g(-7))}", "#7DB56B", g(1.3))
            s += _path(f"M{_f(cx + g(1))} {_f(cy)} q {_f(g(1))} {_f(g(-3))} {_f(g(2))} {_f(g(-5.5))}", "#6AA65C", g(1.3))
            return s
        s += _line(cx, cy, cx, cy - g(4), "#4C8F48", g(1))
        s += _ell(cx - g(2.4), cy - g(4.6), g(2.6), g(1.3), "#86CC70", -20) + _ell(cx + g(2.4), cy - g(4.8), g(2.6), g(1.3), "#62AE56", 20)
        return s
    # ---- jeune pousse ----
    if stage == 1:
        s += shadow(cx, cy, 6, k) + mound(cx, cy, g)
        if kind == "carotte":
            for a in (-30, -8, 14, 34):
                s += _line(cx, cy, cx + math.sin(math.radians(a)) * g(9), cy - math.cos(math.radians(a)) * g(9), "#6DB24D", g(0.8))
                s += leaf(cx + math.sin(math.radians(a)) * g(6), cy - math.cos(math.radians(a)) * g(6), a, g(3.5), g(2.4), "#9AD46E", "#78BC57")
        elif kind in ("courge", "concombre", "courgette"):
            s += _line(cx, cy, cx, cy - g(5), "#4C8F48", g(1.2))
            s += _ell(cx - g(3.5), cy - g(5.5), g(3.6), g(2), "#8ACB6C", -15) + _ell(cx + g(3.5), cy - g(5.8), g(3.6), g(2), "#67AE58", 15)
            s += lobed(cx + g(1), cy - g(9), g(4.8), "#79BE62", "#5C9F4F", VEIN, g)
        elif kind in ("poireau", "oignon"):
            for dx, h, c in ((-2.5, 11, "#7DB56B"), (0, 14, "#8BC27A"), (2.5, 10, "#6AA65C")):
                s += _path(f"M{_f(cx)} {_f(cy)} Q {_f(cx + g(dx * 0.4))} {_f(cy - g(h * 0.6))} {_f(cx + g(dx))} {_f(cy - g(h))}", c, g(1.6))
        elif kind == "haricot":
            s += _line(cx, cy, cx, cy - g(10), "#4C8F48", g(1.2))
            s += trifoliate(cx, cy - g(8), -1, g, "#86CC70", "#62AE56", VEIN, 4.2) + trifoliate(cx, cy - g(10), 1, g, "#86CC70", "#62AE56", VEIN, 4)
        else:
            s += _line(cx, cy, cx, cy - g(7), "#4C8F48", g(1.1))
            for a, h in ((-55, 3), (50, 4), (-25, 6.5), (30, 7)):
                s += leaf(cx, cy - g(h), a, g(5.5), g(3.6), G1, G2, VEIN, g(0.45))
        return s
    # ---- adult / harvest ----
    harvest = stage >= 3
    if kind == "tomate":
        s += shadow(cx, cy, 12, k)
        s += _line(cx + g(6), cy + g(1), cx + g(6), cy - g(48), "#9C6B3E", g(1.8))
        s += _path(f"M{_f(cx)} {_f(cy)} Q {_f(cx - g(2))} {_f(cy - g(22))} {_f(cx + g(1))} {_f(cy - g(44))}", "#4E8F45", g(2))
        for h, a, L in ((6, -62, 14), (9, 58, 15), (16, -52, 15), (19, 62, 14), (26, -48, 13), (29, 55, 13), (35, -35, 11), (38, 30, 10), (43, -12, 7)):
            bx = cx - g(2) + g(h * 0.07)
            by = cy - g(h)
            s += leaf(bx, by, a, g(L), g(L * 0.5), "#6CBB62", "#4C944D", VEIN, g(0.6))
            s += leaf(bx + math.sin(math.radians(a)) * g(L * 0.45), by - math.cos(math.radians(a)) * g(L * 0.45), a + (25 if a > 0 else -25), g(L * 0.45), g(L * 0.3), "#7CC66E", "#589F55")
        s += _line(cx + g(6), cy - g(14), cx + g(1), cy - g(14), "#E6D3A8", g(1.1)) + _line(cx + g(6), cy - g(30), cx + g(1), cy - g(30), "#E6D3A8", g(1.1))
        trusses = ((cx - g(8), cy - g(12)), (cx + g(9), cy - g(22)), (cx - g(6), cy - g(31)))
        cols = (("#E5533D", "#E86A45", "#F08A4B") if harvest else ("#9CCB6A", "#B4D66F", "#E9A35A"))
        for (tx, ty), c in zip(trusses, cols):
            s += _line(tx + g(2), ty - g(4), tx, ty, "#4E8F45", g(0.8))
            s += tomato(tx - g(2), ty + g(1), g(3.1 if harvest else 2.4), c, g) + tomato(tx + g(2.4), ty + g(2), g(2.8 if harvest else 2.2), cols[0] if harvest else "#A9D06C", g)
            if harvest:
                s += tomato(tx, ty + g(5), g(2.6), c, g)
    elif kind == "basilic":
        s += shadow(cx, cy, 8, k)
        s += _line(cx, cy, cx, cy - g(16), "#4C8F48", g(1.4))
        for h, L in ((2, 7), (6.5, 6.5), (11, 5.5), (14.5, 4.2)):
            s += _ell(cx - g(L * 0.55), cy - g(h + 1.5), g(L * 0.62), g(L * 0.36), "#7CCB6E", -32) + _ell(cx + g(L * 0.55), cy - g(h + 1.5), g(L * 0.62), g(L * 0.36), "#5AAE56", 32)
            s += _line(cx - g(L * 0.2), cy - g(h + 1.2), cx - g(L * 0.85), cy - g(h + 2.2), VEIN, g(0.4)) + _line(cx + g(L * 0.2), cy - g(h + 1.2), cx + g(L * 0.85), cy - g(h + 2.2), VEIN, g(0.4))
        if harvest:
            for i in range(4):
                s += _circle(cx, cy - g(17 + i * 2.2), g(1.2 - i * 0.15), "#F6F1E6")
        else:
            s += leaf(cx, cy - g(15), -10, g(4), g(2.6), "#94D884", "#6CBB62")
    elif kind in ("courge", "courgette"):
        s += shadow(cx, cy, 18, k)
        s += _path(f"M{_f(cx - g(14))} {_f(cy + g(2))} Q {_f(cx)} {_f(cy - g(4))} {_f(cx + g(15))} {_f(cy)}", "#5C9A4C", g(1.6))
        s += lobed(cx - g(9), cy - g(4), g(9), "#6FB25C", "#518F47", VEIN, g, 8)
        s += lobed(cx + g(9), cy - g(6), g(10), "#7ABD66", "#5A9A4E", VEIN, g, -12)
        s += lobed(cx, cy - g(11), g(8.5), "#82C46C", "#5F9F51", VEIN, g, 20)
        s += _path(f"M{_f(cx + g(15))} {_f(cy)} c {_f(g(3))} {_f(g(-1))} {_f(g(3))} {_f(g(-5))} {_f(g(0.5))} {_f(g(-5))} c {_f(g(-2))} 0 {_f(g(-2))} {_f(g(2.5))} 0 {_f(g(2.5))}", "#6FAE58", g(0.7))
        if harvest and kind == "courgette":
            for fx, fy, ang in ((cx - g(4), cy + g(3), -20), (cx + g(5), cy + g(2), 15)):
                s += _ell(fx, fy, g(5.5), g(1.9), "#3F8F4A", ang) + _ell(fx - g(1), fy - g(0.6), g(3.5), g(0.7), "#7CC46A", ang, ' fill-opacity="0.6"')
        elif harvest:
            fx, fy = cx - g(2), cy + g(4)
            s += _ell(fx + g(4.5), fy - g(0.5), g(4.2), g(2.2), "#E9B872", -14)
            s += _ell(fx - g(1.5), fy + g(0.8), g(5.4), g(4.2), "#E3A85E", -14)
            s += _ell(fx - g(0.2), fy + g(1.6), g(3.6), g(2.8), "#CF8F45", -14, ' fill-opacity="0.55"')
            s += _circle(fx - g(3), fy - g(1), g(1.2), "#FFFFFF", ' fill-opacity="0.5"')
            s += _line(fx + g(8.4), fy - g(1.4), fx + g(10), fy - g(2.4), "#7A5A2A", g(1.2))
        else:
            s += flower5(cx + g(3), cy - g(16), g(2.2), "#F6C443", "#E08A1E", g)
    elif kind == "carotte":
        s += shadow(cx, cy, 8, k)
        for a, L in ((-38, 15), (-20, 19), (-3, 21), (15, 19), (34, 15)):
            ra = math.radians(a)
            ex, ey = cx + math.sin(ra) * g(L), cy - math.cos(ra) * g(L)
            s += _line(cx, cy, ex, ey, "#5FA445", g(0.9))
            for t in (0.45, 0.65, 0.85):
                px, py = cx + math.sin(ra) * g(L * t), cy - math.cos(ra) * g(L * t)
                sz = 4.2 * (1.1 - t * 0.5)
                s += leaf(px, py, a - 48, g(sz), g(sz * 0.65), "#9AD46E", "#7CBF5A") + leaf(px, py, a + 48, g(sz), g(sz * 0.65), "#8ACB62", "#6CB04D")
        if harvest:
            s += _poly([(cx - g(3.4), cy - g(1)), (cx + g(3.4), cy - g(1)), (cx + g(2.4), cy + g(3)), (cx - g(2.4), cy + g(3))], "#EF8A2E")
            s += _poly([(cx, cy - g(1)), (cx + g(3.4), cy - g(1)), (cx + g(2.4), cy + g(3)), (cx, cy + g(3))], "#D9731F")
            s += _line(cx - g(2.6), cy + g(0.6), cx + g(2.6), cy + g(0.6), "#C9661A", g(0.5)) + _line(cx - g(2.2), cy + g(1.9), cx + g(2.2), cy + g(1.9), "#C9661A", g(0.5))
        else:
            s += _ell(cx, cy, g(2.6), g(1.2), "#EF8A2E")
    elif kind == "oignon":
        s += shadow(cx, cy, 7, k)
        for dx, h, c in ((-5, 20, "#7DB56B"), (-1.5, 25, "#8BC27A"), (2.5, 22, "#6AA65C"), (6, 16, "#79B168")):
            s += _path(f"M{_f(cx)} {_f(cy - g(4))} Q {_f(cx + g(dx * 0.3))} {_f(cy - g(h * 0.7))} {_f(cx + g(dx))} {_f(cy - g(h))}", c, g(2.1))
        bulb_r = 5.4 if harvest else 4.2
        s += _ell(cx, cy - g(1.5), g(bulb_r), g(bulb_r * 0.82), "#E2AE6C")
        s += _ell(cx + g(bulb_r * 0.35), cy - g(1.3), g(bulb_r * 0.6), g(bulb_r * 0.75), "#C98F4E", 0, ' fill-opacity="0.65"')
        s += _path(f"M{_f(cx - g(bulb_r * 0.4))} {_f(cy - g(bulb_r + 0.5))} Q {_f(cx - g(bulb_r * 0.7))} {_f(cy - g(1.5))} {_f(cx - g(bulb_r * 0.35))} {_f(cy + g(2.4))}", "#B97D3E", g(0.5))
        s += _path(f"M{_f(cx + g(bulb_r * 0.3))} {_f(cy - g(bulb_r + 0.5))} Q {_f(cx + g(bulb_r * 0.75))} {_f(cy - g(1.5))} {_f(cx + g(bulb_r * 0.35))} {_f(cy + g(2.4))}", "#B97D3E", g(0.5))
        if not harvest:
            s += _ell(cx, cy + g(1.8), g(bulb_r + 1.2), g(1.6), "#8E5F3D")
    elif kind == "haricot":
        s += shadow(cx, cy, 8, k)
        s += _line(cx, cy + g(1), cx, cy - g(46), "#A07448", g(1.6))
        s += _path(f"M{_f(cx)} {_f(cy)} " + " ".join(f"Q {_f(cx + (g(4) if i % 2 == 0 else g(-4)))} {_f(cy - g(i * 8 + 4))} {_f(cx)} {_f(cy - g(i * 8 + 8))}" for i in range(5)), "#5FA445", g(1))
        for i, h in enumerate((7, 13, 19, 25, 31, 37)):
            side = -1 if i % 2 == 0 else 1
            s += trifoliate(cx + side * g(1.5), cy - g(h), side, g, "#86CC70", "#5FA852", VEIN, 5.6)
        if harvest:
            for x0, y0 in ((-4, 12), (4, 18), (-4, 24), (4, 30)):
                s += _path(f"M{_f(cx + g(x0 * 0.5))} {_f(cy - g(y0))} q {_f(g(x0 * 0.35))} {_f(g(4))} {_f(g(x0 * 0.15))} {_f(g(8))}", "#9BCB45", g(1.7))
        else:
            for x0, y0 in ((-5, 16), (5, 23), (-4, 31)):
                s += _circle(cx + g(x0), cy - g(y0), g(1.3), "#F4EEF8") + _circle(cx + g(x0) + g(0.8), cy - g(y0) - g(0.5), g(0.9), "#D9C9EE")
    elif kind == "fraisier":
        s += shadow(cx, cy, 10, k)
        for a in (-55, -15, 25, 60):
            ra = math.radians(a)
            ex, ey = cx + math.sin(ra) * g(7), cy - math.cos(ra) * g(5)
            s += _line(cx, cy, ex, ey, "#4E8F45", g(0.8))
            s += trifoliate(ex, ey, 0, g, "#5BA85A", "#3F8A47", "#2F6D35", 5)
        if harvest:
            s += strawberry(cx - g(5), cy + g(0.5), g) + strawberry(cx + g(4.5), cy - g(1), g)
        else:
            s += flower5(cx - g(3), cy - g(8), g(1.5), "#FFFFFF", "#F2C232", g) + flower5(cx + g(4), cy - g(6), g(1.4), "#FFFFFF", "#F2C232", g)
    elif kind == "laitue":
        s += shadow(cx, cy, 10, k)
        for i, (a, rx, col) in enumerate(((-150, 7, "#86C05A"), (-30, 7, "#74B04E"), (-110, 7.5, "#93C962"), (-70, 7.5, "#7FB955"),
                                          (-140, 5.5, "#A4D46E"), (-40, 5.5, "#92C863"), (-90, 5, "#B2DC7C"))):
            ra = math.radians(a)
            ox, oy = math.cos(ra) * g(rx * 0.55), math.sin(ra) * g(rx * 0.35)
            s += _ell(cx + ox, cy + oy - g(3 + i * 0.6), g(rx * 0.8), g(rx * 0.55), col, a + 90)
        s += _ell(cx, cy - g(7), g(3.6 if harvest else 2.6), g(3 if harvest else 2.2), "#CFEA9C")
        s += _path(f"M{_f(cx - g(2))} {_f(cy - g(6.5))} q {_f(g(2))} {_f(g(-1.5))} {_f(g(4))} 0", "#9CCB6A", g(0.5))
    elif kind == "poireau":
        s += shadow(cx, cy, 6, k)
        s += _poly([(cx - g(2), cy), (cx + g(2), cy), (cx + g(1.8), cy - g(13)), (cx - g(1.8), cy - g(13))], "#F1F5E0")
        s += _poly([(cx, cy), (cx + g(2), cy), (cx + g(1.8), cy - g(13)), (cx, cy - g(13))], "#DCE7C6")
        for i, (a, L) in enumerate(((-38, 16), (32, 17), (-22, 19), (18, 20), (-6, 18))):
            s += leaf(cx, cy - g(11 + i * 1.5), a, g(L), g(3.6), "#7DBE92", "#5A9F73", "#467F5B", g(0.5))
    return s
