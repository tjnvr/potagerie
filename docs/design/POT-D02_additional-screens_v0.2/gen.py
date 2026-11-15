#!/usr/bin/env python3
"""Generates the .dc.html artboards of the Potagerie canvas — additional V1 screens (D02).
Dates and quantities checked by a separate script (S03); see the README.
The crop drawings come from plants2.py, copied from D01 (see the README)."""
import json, os, sys, datetime

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "project")
os.makedirs(ROOT, exist_ok=True)
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from plants2 import plant  # noqa: E402

# ---------- Palette (same as D01) ----------
C = dict(
    bg="#EEF2E6", card="#E3ECDD", ink="#1F3A2E", muted="#56705F", line="#D3DECD",
    white="#FFFFFF", accent="#F2A65A", accentInk="#5A3410", green="#2F7D5B",
    grassTop="#9CCB86", grassL="#7FB36E", grassR="#6A9F5E",
    bedTop="#A0704A", bedFurrow="#8E5F3D", bedL="#7E5236", bedR="#6B432B",
    woodTop="#D9A86E", woodL="#C08A53", woodR="#A87442",
    crit="#C8443C", warn="#D9822B", good="#2F8A5F",
)

FONTS = '<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link href="https://fonts.googleapis.com/css2?family=Fredoka:wght@500;600&amp;family=Figtree:wght@400;500;600;700&amp;display=swap" rel="stylesheet">'

LOGO = (f'<svg width="34" height="34" viewBox="0 0 34 34" aria-hidden="true">'
        f'<polygon points="17,4 6,12 7,22 17,30" fill="#6BBA62"></polygon><polygon points="17,4 28,12 27,22 17,30" fill="#4A9450"></polygon>'
        f'<polygon points="17,4 11,10 17,15 23,10" fill="#8ED17A"></polygon><circle cx="21" cy="21" r="4" fill="#E5533D"></circle><circle cx="19.8" cy="19.8" r="1.2" fill="#fff" fill-opacity="0.6"></circle></svg>')

H = "font-family: Fredoka, Figtree, sans-serif; font-weight: 600; margin: 0"
ROOTSTYLE = f'font-family: Figtree, system-ui, sans-serif; color: {C["ink"]}; background: {C["bg"]}'


def icon(name, size=18, color=None):
    color = color or C["ink"]
    paths = {
        "plus": f'<path d="M10 4v12M4 10h12" stroke="{color}" stroke-width="2" stroke-linecap="round" fill="none"></path>',
        "check": f'<path d="M4.5 10.5l3.5 3.5 7.5-8" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "down": f'<path d="M5.5 8l4.5 4.5L14.5 8" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "export": f'<path d="M10 3v9M6.5 6.5L10 3l3.5 3.5M4 12.5V16h12v-3.5" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "upload": f'<path d="M10 16V7M6.5 10.5L10 7l3.5 3.5M4 16.5V13" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "garden": f'<path d="M3 12l7-4 7 4-7 4z" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" fill="none"></path><path d="M10 8V4.5M8 6l2-1.5L12 6" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "bars": f'<path d="M3 5h8M6 10h10M3 15h6" stroke="{color}" stroke-width="2" stroke-linecap="round" fill="none"></path>',
        "book": f'<path d="M4 4h5a2 2 0 0 1 2 2v10a2 2 0 0 0-2-2H4zM16 4h-5" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" stroke-linecap="round" fill="none"></path><path d="M16 4v10h-5" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" fill="none"></path>',
        "close": f'<path d="M5 5l10 10M15 5L5 15" stroke="{color}" stroke-width="2" stroke-linecap="round" fill="none"></path>',
        "cal": f'<rect x="3" y="4.5" width="14" height="12" rx="2.5" fill="none" stroke="{color}" stroke-width="1.8"></rect><path d="M3 8.5h14M7 3v3M13 3v3" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
        "move": f'<path d="M10 2.5v15M2.5 10h15M10 2.5l-2.5 2.5M10 2.5l2.5 2.5M10 17.5l-2.5-2.5M10 17.5l2.5-2.5M2.5 10L5 7.5M2.5 10L5 12.5M17.5 10L15 7.5M17.5 10L15 12.5" stroke="{color}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "trash": f'<path d="M4.5 6h11M8 6V4h4v2M6 6l.8 10h6.4L14 6" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "back": f'<path d="M12 4.5L6.5 10l5.5 5.5" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "settings": f'<circle cx="10" cy="10" r="2.6" fill="none" stroke="{color}" stroke-width="1.8"></circle><path d="M10 3.5v2M10 14.5v2M3.5 10h2M14.5 10h2M5.4 5.4l1.4 1.4M13.2 13.2l1.4 1.4M5.4 14.6l1.4-1.4M13.2 6.8l1.4-1.4" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
        "alert": f'<path d="M10 3.5l7.5 13h-15z" stroke="{color}" stroke-width="1.7" stroke-linejoin="round" fill="none"></path><path d="M10 8.5v3.2" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path><circle cx="10" cy="14.3" r="0.9" fill="{color}"></circle>',
        "edit": f'<path d="M4 16l1-4 8.5-8.5a2 2 0 0 1 3 3L8 15z" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" fill="none"></path><path d="M12 5l3 3" stroke="{color}" stroke-width="1.8" stroke-linecap="round"></path>',
        "undo": f'<path d="M7 5L3.5 8.5 7 12M4 8.5h8a4 4 0 0 1 0 8H9" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"></path>',
        "lock": f'<rect x="5" y="9" width="10" height="8" rx="2" fill="none" stroke="{color}" stroke-width="1.8"></rect><path d="M7 9V6.5a3 3 0 0 1 6 0V9" stroke="{color}" stroke-width="1.8" fill="none"></path>',
        "image": f'<rect x="3" y="4" width="14" height="12" rx="2" fill="none" stroke="{color}" stroke-width="1.8"></rect><circle cx="7.5" cy="8.5" r="1.4" fill="{color}"></circle><path d="M4.5 14.5l4-4 2.5 2.5 3-3.5 2.5 3" stroke="{color}" stroke-width="1.6" stroke-linejoin="round" fill="none"></path>',
        "doc": f'<path d="M6 3h6l3 3v11H6z" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" fill="none"></path><path d="M12 3v3h3" stroke="{color}" stroke-width="1.8" stroke-linejoin="round" fill="none"></path>',
    }[name]
    return f'<svg width="{size}" height="{size}" viewBox="0 0 20 20" aria-hidden="true">{paths}</svg>'


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


def crop_icon(kind, stage=2, size=44):
    k = size / 44
    return (f'<svg width="{size}" height="{size}" viewBox="0 0 {size} {size}" aria-hidden="true">'
            f'<ellipse cx="{size / 2:.1f}" cy="{size * 0.8:.1f}" rx="{size * 0.36:.1f}" ry="{size * 0.15:.1f}" fill="{C["bedTop"]}"></ellipse>'
            f'{plant(kind, stage, size / 2, size * 0.8, k * 0.82)}</svg>')


def header_bar(icon_name, title):
    return (f'<span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}; display: inline-flex; align-items: center; gap: 6px">'
            f'{icon(icon_name, 14, C["muted"])}{title}</span>')


def nav_icons(active=None):
    items = [("settings", "Settings", "E10_settings_desktop.dc.html"), ("book", "Catalogue", "E09_catalogue_desktop.dc.html")]
    out = ""
    for ic, label, href in items:
        on = label == active
        out += (f'<a href="{href}" aria-label="{label}" style="display: inline-flex; align-items: center; justify-content: center; width: 40px; height: 40px; border-radius: 50%; '
                f'background: {"#FFFFFF" if not on else C["ink"]}; border: 1px solid {C["line"]}; text-decoration: none">{icon(ic, 17, C["ink"] if not on else "#FFFFFF")}</a>')
    return out


def top_header(title_small, subtitle=None, nav_active=None, back_href=None, close_href=None):
    left = (f'<a href="{back_href}" aria-label="Back" style="width: 40px; height: 40px; border-radius: 50%; border: 1px solid {C["line"]}; background: #FFFFFF; display: grid; place-items: center; text-decoration: none">{icon("back", 17)}</a>'
            if back_href else f'<div style="display: flex; align-items: center; gap: 10px">{LOGO}<span style="{H}; font-size: 22px; color: {C["ink"]}">Potagerie</span></div>')
    titleblk = (f'<div style="display: flex; flex-direction: column; gap: 1px"><strong style="{H}; font-size: 17px">{title_small}</strong>'
                f'{"<span style=" + chr(34) + "font-size: 12.5px; color: " + C["muted"] + chr(34) + ">" + subtitle + "</span>" if subtitle else ""}</div>') if title_small else ""
    right = nav_icons(nav_active)
    if close_href:
        right += f'<a href="{close_href}" aria-label="Close" style="width: 40px; height: 40px; border-radius: 50%; background: {C["bg"]}; display: grid; place-items: center; text-decoration: none">{icon("close", 16)}</a>'
    return (f'<header style="display: flex; align-items: center; gap: 14px; padding: 18px 28px">{left}{titleblk}<div style="flex: 1"></div>{right}</header>')


# ---------- Flat plan (top view, not isometric) ----------
GW, GH = 18, 13  # 9 x 6.5 m, case 50 cm

PLANCHES = [
    dict(id="A", x=0, y=0, w=6, h=2, kind="bed", nom="Back of the garden"),
    dict(id="D", x=7, y=0, w=6, h=2, kind="bed", nom="Roots"),
    dict(id="B", x=0, y=3, w=8, h=2, kind="bed", nom="The tomatoes"),
    dict(id="E", x=9, y=3, w=8, h=2, kind="bed", nom="Shed side"),
    dict(id="C", x=0, y=6, w=8, h=2, kind="bed", nom="The zucchinis"),
    dict(id="F", x=9, y=6, w=4, h=4, kind="bac", nom="Leek square"),
    dict(id="G", x=14, y=6, w=3, h=4, kind="bac", nom="Salad square"),
    dict(id="H", x=0, y=9, w=6, h=2, kind="butte", nom="End of the aisle"),
]


def flat_plan(cell=26, planches=None, gw=GW, gh=GH, highlight=None, free=None, conflict=None, drag_to=None, labels=True):
    planches = planches if planches is not None else PLANCHES
    W, Hh = gw * cell, gh * cell
    s = [f'<rect x="0" y="0" width="{W}" height="{Hh}" rx="12" fill="{C["grassTop"]}"></rect>']
    for i in range(1, gw):
        s.append(f'<line x1="{i * cell}" y1="0" x2="{i * cell}" y2="{Hh}" stroke="#FFFFFF" stroke-opacity="{0.5 if i % 2 == 0 else 0.22}" stroke-width="1"></line>')
    for j in range(1, gh):
        s.append(f'<line x1="0" y1="{j * cell}" x2="{W}" y2="{j * cell}" stroke="#FFFFFF" stroke-opacity="{0.5 if j % 2 == 0 else 0.22}" stroke-width="1"></line>')
    for p in planches:
        x, y, w, h = p["x"] * cell, p["y"] * cell, p["w"] * cell, p["h"] * cell
        bac, butte = p["kind"] == "bac", p["kind"] == "butte"
        fill = C["woodTop"] if bac else C["bedTop"]
        in_conflict = conflict and p["id"] in conflict
        if in_conflict:
            fill = "#E79B93"
        s.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="5" fill="{fill}"{" fill-opacity=\"0.8\"" if in_conflict else ""}></rect>')
        if bac:
            s.append(f'<rect x="{x + 5}" y="{y + 5}" width="{w - 10}" height="{h - 10}" rx="3" fill="{C["bedTop"]}"></rect>')
        elif butte:
            s.append(f'<path d="M{x + 4} {y + h - 2} Q {x + w / 2} {y - 3} {x + w - 4} {y + h - 2}" fill="none" stroke="{C["bedFurrow"]}" stroke-width="1.5"></path>')
        else:
            for k in range(1, int(p["h"] * 2)):
                yy = y + k * cell / 2
                s.append(f'<line x1="{x + 6}" y1="{yy}" x2="{x + w - 6}" y2="{yy}" stroke="{C["bedFurrow"]}" stroke-width="1.2" stroke-linecap="round"></line>')
        if labels and w > cell * 1.2:
            label = p["nom"]
            lw = len(label) * 6.2 + 14
            s.append(f'<rect x="{x + 5}" y="{y + 5}" width="{lw}" height="18" rx="9" fill="#FFFFFF" fill-opacity="0.92"></rect>'
                      f'<text x="{x + 5 + lw / 2:.1f}" y="{y + 17.5:.1f}" text-anchor="middle" font-family="Figtree, sans-serif" font-weight="600" font-size="10.5" fill="{C["ink"]}">{label}</text>')
        if p["id"] == highlight:
            s.append(f'<rect x="{x - 2.5}" y="{y - 2.5}" width="{w + 5}" height="{h + 5}" rx="7" fill="none" stroke="{C["accent"]}" stroke-width="2.5"></rect>')
        if p["id"] == free:
            s.append(f'<rect x="{x + 3}" y="{y + 3}" width="{w - 6}" height="{h - 6}" rx="4" fill="none" stroke="#FFFFFF" stroke-width="1.6" stroke-dasharray="5 4"></rect>')
        if drag_to and p["id"] == drag_to[0]:
            dx, dy = drag_to[1] * cell, drag_to[2] * cell
            s.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="5" fill="none" stroke="#FFFFFF" stroke-width="2" stroke-dasharray="6 5"></rect>')
            s.append(f'<rect x="{dx}" y="{dy}" width="{w}" height="{h}" rx="5" fill="{C["crit"]}" fill-opacity="0.55" stroke="{C["crit"]}" stroke-width="2.5"></rect>')
    body = "".join(s)
    return f'<svg viewBox="0 0 {W} {Hh}" width="100%" style="display: block; max-width: {W}px" role="img" aria-label="Top view of the garden plan">{body}</svg>'


# ---------- phase bar (flat timeline, reused for E06/E07/E11) ----------
def mix(hex_, t):
    h = hex_.lstrip("#")
    r, g, b = int(h[:2], 16), int(h[2:4], 16), int(h[4:], 16)
    r, g, b = [round(v + (255 - v) * t) for v in (r, g, b)]
    return f"#{r:02X}{g:02X}{b:02X}"


CROPCOL = dict(tomate="#E5533D", laitue="#92C65E", carotte="#EF8A2E", haricot="#6DB24D", courgette="#3F8F4A", poireau="#6FB487")


def phase_bar(abri_d, grow_d, harvest_d, col, height=14):
    segs = ""
    if abri_d > 0:
        segs += f'<div style="flex: {abri_d} 1 0; background: repeating-linear-gradient(135deg, {mix(col, 0.55)} 0 4px, {mix(col, 0.85)} 4px 8px)"></div>'
    if grow_d > 0:
        segs += f'<div style="flex: {grow_d} 1 0; background: {mix(col, 0.45)}"></div>'
    segs += f'<div style="flex: {harvest_d} 1 0; background: {col}"></div>'
    return f'<div style="display: flex; height: {height}px; border-radius: {height // 2}px; overflow: hidden">{segs}</div>'


# ---------- form components ----------
def field(lid, lbl, val, unit=""):
    return (f'<label for="{lid}" style="flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; font-size: 12px; font-weight: 700; color: {C["muted"]}">{lbl}'
            f'<span style="display: flex; align-items: center; gap: 6px; padding: 0 12px; height: 46px; border-radius: 12px; background: {C["bg"]}">'
            f'<input id="{lid}" value="{val}" style="border: 0; background: transparent; font: 600 16px Figtree, sans-serif; color: {C["ink"]}; width: 100%; min-width: 0">'
            f'<span style="font-size: 13px; color: {C["muted"]}">{unit}</span></span></label>')


def pill_group(label, options, sel_idx, name=""):
    btns = "".join(
        f'<button aria-pressed="{"true" if i == sel_idx else "false"}" style="flex: 1; border: 0; padding: 9px 4px; border-radius: 99px; font: 600 13.5px Figtree, sans-serif; cursor: pointer; '
        f'{"background: " + C["ink"] + "; color: #FFFFFF" if i == sel_idx else "background: transparent; color: " + C["muted"]}">{t}</button>'
        for i, t in enumerate(options))
    return (f'<div style="display: flex; flex-direction: column; gap: 6px"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">{label}</span>'
            f'<div role="radiogroup" aria-label="{name or label}" style="display: flex; padding: 4px; border-radius: 99px; background: {C["bg"]}">{btns}</div></div>')


def card(content, pad="24px", radius="28px", bg="#FFFFFF"):
    return f'<section style="background: {bg}; border-radius: {radius}; padding: {pad}">{content}</section>'


def primary_btn(label, href=None, ic=None, bg=None, fg="#FFFFFF", block=False):
    tag = "a" if href else "button"
    attrs = f'href="{href}"' if href else ""
    style = (f'display: inline-flex; {"width: 100%; " if block else ""}justify-content: center; align-items: center; gap: 8px; background: {bg or C["green"]}; '
             f'color: {fg}; text-decoration: none; border: 0; border-radius: 99px; padding: 14px 22px; font: 600 15.5px Figtree, sans-serif; cursor: pointer')
    return f'<{tag} {attrs} style="{style}">{icon(ic, 18, fg) if ic else ""}{label}</{tag}>'


def secondary_btn(label, href=None, ic=None, fg=None):
    tag = "a" if href else "button"
    attrs = f'href="{href}"' if href else ""
    fg = fg or C["ink"]
    style = (f'display: inline-flex; justify-content: center; align-items: center; gap: 8px; background: #FFFFFF; border: 1px solid {C["line"]}; '
             f'color: {fg}; text-decoration: none; border-radius: 99px; padding: 13px 20px; font: 600 15px Figtree, sans-serif; cursor: pointer')
    return f'<{tag} {attrs} style="{style}">{icon(ic, 17, fg) if ic else ""}{label}</{tag}>'


# ================= Artboard: E02 · Plot creation (computer, Main.dc.html) =================
def e02_board():
    steps = "".join(
        f'<span style="display: inline-flex; align-items: center; gap: 6px; font-size: 13px; font-weight: 600; color: {C["ink"] if on else C["muted"]}">'
        f'<span style="width: 22px; height: 22px; border-radius: 50%; display: grid; place-items: center; font-size: 12px; font-weight: 700; '
        f'background: {C["ink"] if on else "#FFFFFF"}; color: {"#FFFFFF" if on else C["muted"]}; border: 1px solid {C["line"]}">{n}</span>{t}</span>'
        for n, t, on in ((1, "Plot", True), (2, "Beds", False)))
    header = (f'<header style="display: flex; align-items: center; gap: 18px; padding: 20px 32px">'
              f'<div style="display: flex; align-items: center; gap: 10px">{LOGO}<span style="{H}; font-size: 24px">Potagerie</span></div>'
              f'<div style="flex: 1"></div><div style="display: flex; gap: 20px">{steps}</div></header>')

    form = card(
        f'<div style="display: flex; flex-direction: column; gap: 22px">'
        f'<div style="display: flex; flex-direction: column; gap: 4px"><h1 style="{H}; font-size: 26px">Create your plot</h1>'
        f'<p style="margin: 0; font-size: 14px; color: {C["muted"]}">The whole rectangular area that you will then cut into beds in the editor.</p></div>'
        f'<div style="display: flex; gap: 14px">{field("t-l", "Length", "9,0", "m")}{field("t-w", "Width", "6,5", "m")}</div>'
        f'{pill_group("Cell size", ["25 cm", "50 cm", "100 cm"], 1, "Cell size")}'
        f'<p style="margin: -8px 0 0; display: flex; gap: 8px; font-size: 13px; color: {C["muted"]}; align-items: flex-start">{icon("lock", 15, C["muted"])}'
        f'<span>The cell size sets the size of the beds’ segments: it cannot be changed once the plot is created.</span></p>'
        f'{pill_group("Minimum aisle width", ["30 cm", "50 cm", "80 cm"], 1, "Aisle width")}'
        f'<p style="margin: -8px 0 0; font-size: 13px; color: {C["muted"]}">Kept between the beds in the editor. Can be changed at any time in the settings.</p>'
        f'<div style="display: flex; gap: 12px; padding-top: 6px">{primary_btn("Create the plot", "../../POT-D01_ecrans-cles_v1.1/project/E03_plan-editor_desktop.dc.html", "check")}</div>'
        f'</div>')

    preview = card(
        f'<div style="display: flex; flex-direction: column; gap: 14px">'
        f'<div style="display: flex; justify-content: space-between; align-items: baseline"><strong style="font-size: 14px">Grid preview</strong>'
        f'<span style="font-size: 13px; color: {C["muted"]}">9.0 × 6.5 m · case 50 cm · 18 × 13 cases</span></div>'
        f'<div style="display: flex; justify-content: center">{flat_plan(cell=26, planches=[], labels=False)}</div>'
        f'<p style="margin: 0; font-size: 13px; color: {C["muted"]}">The plot is still empty: you will drag the first beds onto it in the plan editor.</p>'
        f'</div>', bg=C["card"])

    body = (f'<div style="{ROOTSTYLE}; min-height: 900px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">'
            f'<div style="flex: 1 1 460px; max-width: 520px">{form}</div>'
            f'<div style="flex: 1 1 460px; max-width: 560px">{preview}</div></div></div>')
    return page("Plot creation — computer", body, {"$preview": {"width": 1320, "height": 900}})


# ================= Artboard : E06 · Bed panel (ordinateur) =================
def series_row(kind, nom, qty_txt, dates_txt, stage_txt, stage_bg=None):
    return (f'<div style="display: flex; gap: 12px; align-items: flex-start; padding: 12px; border-radius: 18px; background: {C["bg"]}">'
            f'<span style="flex: none; width: 48px; height: 48px; border-radius: 13px; background: #FFFFFF; display: grid; place-items: center">{crop_icon(kind, 2, 40)}</span>'
            f'<div style="display: flex; flex-direction: column; gap: 3px; min-width: 0"><div style="display: flex; align-items: center; gap: 8px"><strong style="font-size: 15px">{nom}</strong>'
            f'<span style="font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 99px; background: {stage_bg or "#FFFFFF"}; color: {C["green"] if not stage_bg else C["accentInk"]}">{stage_txt}</span></div>'
            f'<span style="font-size: 13px; color: {C["muted"]}">{qty_txt}</span><span style="font-size: 13px">{dates_txt}</span></div></div>')


def e06_board():
    # dimmed backdrop (Garden screen, Roots bed highlighted)
    backdrop = (
        f'<div style="position: relative; flex: 1 1 820px; min-width: 0; background: {C["card"]}; border-radius: 32px; padding: 22px; filter: saturate(0.85); opacity: 0.55">'
        f'<div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px">'
        f'<div style="display: flex; align-items: center; gap: 10px">{LOGO}<span style="{H}; font-size: 22px">Potagerie</span></div>'
        f'<span style="{H}; font-size: 20px">20 Jun 2027</span></div>'
        f'<div style="display: flex; justify-content: center">{flat_plan(cell=30, highlight="D")}</div>'
        f'</div>'
        f'<div aria-hidden="true" style="position: absolute; inset: 0; background: rgba(31,58,46,0.12); border-radius: 32px"></div>')

    hist = (f'<details open style="display: flex; flex-direction: column; gap: 8px">'
            f'<summary style="cursor: pointer; list-style: none; display: flex; justify-content: space-between; align-items: center; font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">'
            f'<span style="display: inline-flex; align-items: center; gap: 6px">{icon("lock", 13, C["muted"])}History · read only</span>{icon("down", 14, C["muted"])}</summary>'
            f'<div style="display: flex; gap: 12px; align-items: flex-start; padding: 12px; border-radius: 18px; background: {C["bg"]}; opacity: 0.85">'
            f'<span style="flex: none; width: 48px; height: 48px; border-radius: 13px; background: #FFFFFF; display: grid; place-items: center">{crop_icon("carotte", 3, 40)}</span>'
            f'<div style="display: flex; flex-direction: column; gap: 3px"><strong style="font-size: 14px">2026 · Carrot</strong>'
            f'<span style="font-size: 13px; color: {C["muted"]}">15 Mar → 2 Aug · ≈ 148 plants (4 rows × 37)</span></div></div>'
            f'</details>')

    actions = (
        f'<div style="display: flex; gap: 10px; flex-wrap: wrap; margin-top: auto">'
        f'<a href="E07_planting_desktop.dc.html" style="flex: 1 1 130px; display: inline-flex; justify-content: center; align-items: center; gap: 8px; background: {C["green"]}; color: #FFFFFF; text-decoration: none; border-radius: 99px; padding: 12px 14px; font: 600 14.5px Figtree, sans-serif">{icon("plus", 17, "#FFFFFF")}Plant here</a>'
        f'<a href="../../POT-D01_ecrans-cles_v1.1/project/E03_plan-editor_desktop.dc.html" style="flex: 1 1 110px; display: inline-flex; justify-content: center; align-items: center; gap: 8px; border: 1px solid {C["line"]}; background: #FFFFFF; color: {C["ink"]}; text-decoration: none; border-radius: 99px; padding: 12px 14px; font: 600 14.5px Figtree, sans-serif">{icon("move", 16)}Move</a>'
        f'<button style="flex: 1 1 110px; border: 1px solid {C["line"]}; background: #FFFFFF; color: {C["crit"]}; border-radius: 99px; padding: 12px 14px; font: 600 14.5px Figtree, sans-serif; cursor: pointer; display: inline-flex; justify-content: center; align-items: center; gap: 8px">{icon("trash", 16, C["crit"])}Supprimer</button>'
        f'</div>')

    panel = (
        f'<aside style="flex: 1 1 400px; max-width: 440px; min-width: 0; background: #FFFFFF; border-radius: 28px; padding: 26px; display: flex; flex-direction: column; gap: 18px; box-shadow: 0 20px 48px rgba(31,58,46,0.18)">'
        f'<div style="display: flex; justify-content: space-between; align-items: center; gap: 8px">'
        f'<label for="pl-name" style="flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Bed D</span>'
        f'<input id="pl-name" value="Roots" style="{H}; font-size: 24px; color: {C["ink"]}; border: 0; border-bottom: 2px dashed {C["line"]}; background: transparent; padding: 2px 0; width: 100%"></label>'
        f'<a href="../../POT-D01_ecrans-cles_v1.1/project/Main.dc.html" aria-label="Close the panel" style="width: 36px; height: 36px; border-radius: 50%; background: {C["bg"]}; display: grid; place-items: center; text-decoration: none">{icon("close", 16)}</a></div>'
        f'<div style="display: flex; gap: 8px">{chip("3 × 1.2 m", C["bg"])}{chip("3.6 m²", C["bg"])}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 10px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Plantings of 2027</span>'
        f'{series_row("carotte", "Carrot", "Segments 1–3 · ≈ 148 plants (4 rows × 37)", "15 Mar → harvest from 23 Jun", "Adult")}'
        f'{series_row("poireau", "Leek", "Segments 4–6 · ≈ 48 plants (4 rows × 12)", "Sown under cover on 1 May · transplanting on 10 July", "Sown under cover", "#FFF3E4")}'
        f'</div>'
        f'{hist}'
        f'<div style="display: flex; flex-direction: column; gap: 6px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Before on this bed</span>'
        f'<div style="display: flex; justify-content: space-between; font-size: 14px"><span>2026</span><span style="color: {C["muted"]}">Apiaceae (carrot)</span></div>'
        f'<div style="display: flex; justify-content: space-between; font-size: 14px"><span>2025</span><span style="color: {C["muted"]}">Nothing noted</span></div></div>'
        f'{actions}'
        f'</aside>')

    confirm = (
        f'<div style="position: absolute; inset: 0; background: rgba(31,58,46,0.45); display: flex; align-items: center; justify-content: center; border-radius: 32px">'
        f'<div style="width: 420px; background: #FFFFFF; border-radius: 24px; padding: 26px; display: flex; flex-direction: column; gap: 14px; box-shadow: 0 24px 60px rgba(31,58,46,0.3)">'
        f'<div style="display: flex; gap: 10px; align-items: flex-start">{icon("alert", 22, C["warn"])}<h2 style="{H}; font-size: 19px">Delete Roots?</h2></div>'
        f'<p style="margin: 0; font-size: 14px; line-height: 1.5">If you continue, the bed will be removed from the plan from <strong>2028</strong>.</p>'
        f'<div style="padding: 12px 14px; border-radius: 14px; background: #FDF1E4; font-size: 13.5px; line-height: 1.5">'
        f'<strong>1 planting</strong> will be deleted, because it touches 2028 — including those straddling the year: <strong>Leek</strong> (10 Jul 2027 → 5 Feb 2028).<br>'
        f'La <strong>Carrot</strong>, already finished on 2 Aug 2027, is kept.</div>'
        f'<div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 4px">{secondary_btn("Cancel")}{primary_btn("Delete the bed", ic="trash", bg=C["crit"])}</div>'
        f'</div></div>')

    stage = (f'<div style="flex: 1 1 820px; min-width: 0; position: relative">{backdrop}{confirm}</div>')
    body = (f'<div style="{ROOTSTYLE}; min-height: 760px">'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 28px 32px; align-items: stretch">{stage}{panel}</div>'
            f'</div>')
    return page("Bed panel — computer", body, {"$preview": {"width": 1440, "height": 760}})


# ================= Artboard : E07 · New planting (ordinateur) =================
def e07_board():
    lab = lambda n, t: (f'<div style="display: flex; align-items: center; gap: 10px"><span style="width: 24px; height: 24px; border-radius: 50%; background: {C["ink"]}; color: #FFFFFF; display: grid; place-items: center; font: 700 13px Figtree, sans-serif">{n}</span>'
                        f'<span style="font: 600 17px Fredoka, Figtree, sans-serif; color: {C["ink"]}">{t}</span></div>')
    crops = [("poireau", "Leek", False), ("laitue", "Lettuce", True), ("carotte", "Carrot", False),
             ("haricot", "Green bean", False), ("courgette", "Zucchini", False), ("tomate", "Tomato", False)]
    grid = "".join(
        f'<button aria-pressed="{"true" if sel else "false"}" style="display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 10px 4px 8px; border-radius: 16px; cursor: pointer; '
        f'border: {"2px solid " + C["accent"] if sel else "2px solid transparent"}; background: {"#FFF3E4" if sel else C["bg"]}; font: 600 12px Figtree, sans-serif; color: {C["ink"]}">'
        f'{crop_icon(k, 2, 42)}{n}</button>'
        for k, n, sel in crops)

    ic_plant = ('<svg width="40" height="40" viewBox="0 0 40 40" aria-hidden="true"><path d="M12 25h16l-2 10H14z" fill="#3E6E52"></path>'
                '<path d="M20 25V12" stroke="#4C8F48" stroke-width="2" stroke-linecap="round"></path><ellipse cx="14" cy="17" rx="6" ry="3" fill="#6CBB62" transform="rotate(-30 14 17)"></ellipse><ellipse cx="26" cy="15" rx="6" ry="3" fill="#4C944D" transform="rotate(30 26 15)"></ellipse>'
                '<ellipse cx="20" cy="10" rx="4" ry="2.5" fill="#86CB74"></ellipse><rect x="27" y="27" width="10" height="7" rx="2" fill="#F2A65A"></rect></svg>')
    ic_godet = ('<svg width="40" height="40" viewBox="0 0 40 40" aria-hidden="true"><path d="M11 22h18l-3 13H14z" fill="#C9875A"></path><path d="M11 22h18v3H11z" fill="#B0703F"></path>'
                '<path d="M20 22v-7" stroke="#4C8F48" stroke-width="2" stroke-linecap="round"></path><ellipse cx="16" cy="14" rx="4" ry="2.2" fill="#7CC46A" transform="rotate(-25 16 14)"></ellipse><ellipse cx="24" cy="13" rx="4" ry="2.2" fill="#5FAA55" transform="rotate(25 24 13)"></ellipse></svg>')
    ic_seed = ('<svg width="40" height="40" viewBox="0 0 40 40" aria-hidden="true"><path d="M4 28q16-5 32 0v6H4z" fill="#A0704A"></path>'
               '<ellipse cx="13" cy="20" rx="2.6" ry="1.6" fill="#8A6A3A" transform="rotate(-20 13 20)"></ellipse><ellipse cx="21" cy="16" rx="2.6" ry="1.6" fill="#8A6A3A"></ellipse><ellipse cx="28" cy="21" rx="2.6" ry="1.6" fill="#8A6A3A" transform="rotate(25 28 21)"></ellipse>'
               '<path d="M13 23v3M21 19v6M28 24v3" stroke="#8A6A3A" stroke-width="1" stroke-dasharray="1.5 1.5"></path></svg>')
    modes = [(ic_godet, "I sow under cover", "In pots, then transplanted to the garden", False),
             (ic_plant, "I buy plants", "Ready to plant, from a garden centre", True),
             (ic_seed, "I sow in the open ground", "Directly in the bed", False)]
    mode_cards = "".join(
        f'<button aria-pressed="{"true" if sel else "false"}" style="display: flex; align-items: center; gap: 12px; width: 100%; text-align: left; padding: 11px 14px; border-radius: 16px; cursor: pointer; '
        f'border: {"2px solid " + C["accent"] if sel else "2px solid " + C["line"]}; background: {"#FFF3E4" if sel else "#FFFFFF"}; font: 600 15px Figtree, sans-serif; color: {C["ink"]}">'
        f'<span style="flex: none; width: 46px; height: 46px; border-radius: 12px; background: {"#FFFFFF" if sel else C["bg"]}; display: grid; place-items: center">{ic}</span>'
        f'<span style="display: flex; flex-direction: column; gap: 1px; flex: 1"><span>{t}</span><span style="font-size: 12px; font-weight: 500; color: {C["muted"]}">{sub}</span></span>'
        f'<span aria-hidden="true" style="flex: none; width: 20px; height: 20px; border-radius: 50%; border: 2px solid {C["accent"] if sel else C["line"]}; box-sizing: border-box; display: grid; place-items: center">{"<span style=" + chr(34) + "width: 10px; height: 10px; border-radius: 50%; background: " + C["accent"] + chr(34) + "></span>" if sel else ""}</span></button>'
        for ic, t, sub, sel in modes)

    # mini-timeline: 2 Aug (planting) -> 21 Sep (harvest) -> 6 Oct (shortened end, instead of 11 Oct)
    bar = phase_bar(0, 50, 15, CROPCOL["laitue"], height=16)
    keys = [("2 Aug", "Planting", "flex-start"), ("21 Sep", "First harvest", "center"), ("6 Oct", "Edited end", "flex-end")]
    marks = "".join(
        f'<div style="display: flex; flex-direction: column; align-items: {al}; white-space: nowrap">'
        f'<strong style="font-size: 13px">{d}</strong><span style="font-size: 11px; color: {C["muted"] if t != "Edited end" else C["accentInk"]}; font-weight: {700 if t == "Edited end" else 400}">{t}</span></div>'
        for d, t, al in keys)
    manual_badge = (f'<span style="display: inline-flex; align-items: center; gap: 6px; padding: 3px 10px; border-radius: 99px; background: #FDF1E4; color: {C["accentInk"]}; font-size: 12px; font-weight: 700">'
                    f'{icon("edit", 13, C["accentInk"])}Edited by hand: end shortened to 6 Oct</span>'
                    f'<a href="#" style="font-size: 12px; font-weight: 600">Recompute (11 Oct)</a>')
    timeline = (f'<div style="display: flex; flex-direction: column; gap: 10px; padding: 16px; border-radius: 18px; background: {C["bg"]}">'
                f'{bar}<div style="display: flex; justify-content: space-between">{marks}</div>'
                f'<div style="display: flex; flex-wrap: wrap; gap: 8px; align-items: center; padding-top: 10px; border-top: 1px solid {C["line"]}">'
                f'<span style="font-size: 13px; color: {C["muted"]}">Free slot · 2 Aug → 5 Oct 2027 (65 d)</span>{manual_badge}</div></div>')

    parts = "".join(
        f'<button aria-pressed="{"true" if on else "false"}" style="flex: 1; border: 0; padding: 8px 0; border-radius: 99px; font: 600 13px Figtree, sans-serif; cursor: pointer; '
        f'{"background: " + C["ink"] + "; color: #FFFFFF" if on else "background: transparent; color: " + C["muted"]}">{t}</button>'
        for t, on in (("All", True), ("½", False), ("⅓", False), ("¼", False)))
    tsegs = "".join(
        f'<button aria-pressed="true" aria-label="Segment {i + 1}" style="flex: 1; height: 38px; border: 0; cursor: pointer; background: #B9DCC6; box-shadow: inset 0 0 0 2px {C["green"]}; '
        f'border-radius: {"9px 0 0 9px" if i == 0 else ("0 9px 9px 0" if i == 5 else "0")}"></button>'
        for i in range(6))
    zone = (f'<div style="display: flex; flex-direction: column; gap: 8px">'
            f'<div style="display: flex; justify-content: space-between; align-items: baseline"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">Zone of the bed · End of the aisle</span><span style="font-size: 13px; font-weight: 600">3 m by 1.2 m</span></div>'
            f'<div role="group" aria-label="Zone shortcuts" style="display: flex; padding: 4px; border-radius: 99px; background: {C["bg"]}">{parts}</div>'
            f'<div role="group" aria-label="50 cm segments" style="display: flex; gap: 3px">{tsegs}</div></div>')

    left = card(
        f'<div style="display: flex; flex-direction: column; gap: 18px">'
        f'{lab(1, "Which crop?")}<div style="display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: 6px">{grid}</div>'
        f'{lab(2, "How do you start?")}<div role="radiogroup" aria-label="Start mode" style="display: flex; flex-direction: column; gap: 8px">{mode_cards}</div>'
        f'{lab(3, "When do you plant?")}'
        f'<label for="d-plant" style="display: flex; align-items: center; gap: 10px; padding: 0 14px; height: 50px; border-radius: 14px; background: {C["bg"]}">{icon("cal", 20, C["muted"])}'
        f'<input id="d-plant" value="Monday 2 August 2027" style="border: 0; background: transparent; font: 600 16px Figtree, sans-serif; color: {C["ink"]}; width: 100%"></label>'
        f'</div>')
    right = card(f'<div style="display: flex; flex-direction: column; gap: 18px">{timeline}{zone}'
                 f'<div style="display: flex; justify-content: space-between; align-items: center; padding-top: 10px; border-top: 1px solid {C["line"]}">'
                 f'<span style="font-size: 14px; color: {C["muted"]}">≈ 40 plants, 4 rows × 10</span><strong style="font-size: 14px">1 tray of plants</strong></div>'
                 f'{primary_btn("Plant 40 lettuces", "../../POT-D01_ecrans-cles_v1.1/project/Main.dc.html", "check", block=True)}'
                 f'</div>', bg=C["card"])

    header = top_header("New planting", "End of the aisle", close_href="../../POT-D01_ecrans-cles_v1.1/project/Main.dc.html")
    body = (f'<div style="{ROOTSTYLE}; min-height: 720px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">'
            f'<div style="flex: 1 1 520px; max-width: 620px">{left}</div><div style="flex: 1 1 380px; max-width: 420px">{right}</div></div></div>')
    return page("New planting — computer", body, {"$preview": {"width": 1320, "height": 720}})


# ================= Artboard : E09 · Catalogue (ordinateur) =================
CULTURE_FICHES = {
    "tomate": dict(nom="Tomato", famille="Solanaceae", abri="56 d", apres="75 d", duree="70 d", fen="1 May → 15 Jun", rang="80 cm", pied="60 cm", saison=False),
    "laitue": dict(nom="Lettuce", famille="Asteraceae", abri="28 d", apres="50 d", duree="20 d", fen="1 Mar → 30 Sep", rang="30 cm", pied="30 cm", saison=True),
    "carotte": dict(nom="Carrot", famille="Apiaceae", abri="Not sown under cover", apres="100 d", duree="40 d", fen="1 Mar → 15 Jul", rang="25 cm", pied="4 cm", saison=True),
    "haricot": dict(nom="Green bean", famille="Fabaceae", abri="Not sown under cover", apres="60 d", duree="30 d", fen="10 May → 31 Jul", rang="50 cm", pied="10 cm", saison=False),
    "courgette": dict(nom="Zucchini", famille="Cucurbitaceae", abri="21 d", apres="50 d", duree="80 d", fen="1 May → 30 Jun", rang="100 cm", pied="100 cm", saison=False),
    "poireau": dict(nom="Leek", famille="Amaryllidaceae", abri="70 d", apres="120 d", duree="90 d", fen="15 May → 31 Jul", rang="30 cm", pied="12 cm", saison=False),
}


def e09_board():
    order = ["laitue", "carotte", "poireau", "haricot", "courgette", "tomate"]
    cards = ""
    for k in order:
        f_ = CULTURE_FICHES[k]
        sel = k == "laitue"
        cards += (f'<button aria-pressed="{"true" if sel else "false"}" style="display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 14px 8px; border-radius: 18px; cursor: pointer; text-align: center; '
                  f'border: {"2px solid " + C["accent"] if sel else "2px solid " + C["line"]}; background: {"#FFF3E4" if sel else "#FFFFFF"}">'
                  f'{crop_icon(k, 3, 56)}<strong style="font-size: 14px">{f_["nom"]}</strong><span style="font-size: 11.5px; color: {C["muted"]}">{f_["famille"]}</span>'
                  + (f'<span style="font-size: 10.5px; font-weight: 700; padding: 2px 9px; border-radius: 99px; background: {C["good"]}; color: #FFFFFF">In season</span>' if f_["saison"] else "")
                  + '</button>')
    families = ["All", "Asteraceae", "Apiaceae", "Amaryllidaceae", "Fabaceae", "Cucurbitaceae", "Solanaceae"]
    fam_chips = "".join(
        f'<button aria-pressed="{"true" if i == 0 else "false"}" style="border: 0; padding: 7px 14px; border-radius: 99px; font: 600 13px Figtree, sans-serif; cursor: pointer; '
        f'{"background: " + C["ink"] + "; color: #FFFFFF" if i == 0 else "background: " + C["bg"] + "; color: " + C["muted"]}">{t}</button>'
        for i, t in enumerate(families))

    list_card = card(
        f'<div style="display: flex; flex-direction: column; gap: 16px">'
        f'<div style="display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 10px">'
        f'<h1 style="{H}; font-size: 24px">Catalogue</h1>'
        f'<label style="display: inline-flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 600; color: {C["ink"]}">'
        f'<span style="width: 38px; height: 22px; border-radius: 99px; background: {C["good"]}; position: relative"><span style="position: absolute; top: 2px; right: 2px; width: 18px; height: 18px; border-radius: 50%; background: #FFFFFF"></span></span>'
        f'In season now · 20 April 2027</label></div>'
        f'<div style="display: flex; flex-wrap: wrap; gap: 8px">{fam_chips}</div>'
        f'<div style="display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px">{cards}</div>'
        f'<p style="margin: 0; font-size: 12.5px; color: {C["muted"]}">67 crops in V1 (S02) · longer list after V1.</p>'
        f'</div>')

    f_ = CULTURE_FICHES["laitue"]
    stages = "".join(f'<div style="display: flex; flex-direction: column; align-items: center; gap: 6px"><span style="display: grid; place-items: center; height: 84px; width: 84px; border-radius: 18px; background: {C["card"]}">{crop_icon("laitue", st, 68)}</span>'
                      f'<span style="font-size: 11.5px; color: {C["muted"]}">{n}</span></div>' for st, n in enumerate(["Sowing", "Young shoot", "Adult", "Harvest"]))
    rows = [("Family", f_["famille"]), ("Time under cover", f_["abri"]), ("Until harvest", f_["apres"]),
            ("Harvest duration", f_["duree"]), ("Planting-out window", f_["fen"]), ("Spacing between rows", f_["rang"]), ("Spacing along the row", f_["pied"])]
    table = "".join(f'<div style="display: flex; justify-content: space-between; gap: 12px; padding: 10px 0; border-top: 1px solid {C["line"]}"><span style="font-size: 13.5px; color: {C["muted"]}">{k}</span><strong style="font-size: 13.5px">{v}</strong></div>' for k, v in rows)
    fiche_card = card(
        f'<div style="display: flex; flex-direction: column; gap: 16px">'
        f'<div style="display: flex; justify-content: space-between; align-items: flex-start"><div style="display: flex; flex-direction: column; gap: 2px">'
        f'<span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Sheet</span><h2 style="{H}; font-size: 24px">{f_["nom"]}</h2></div>'
        f'<span style="font-size: 10.5px; font-weight: 700; padding: 4px 10px; border-radius: 99px; background: {C["good"]}; color: #FFFFFF">In season</span></div>'
        f'<div style="display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px">{stages}</div>'
        f'<div>{table}</div>'
        f'<p style="margin: 0; display: flex; gap: 8px; font-size: 12.5px; color: {C["muted"]}; align-items: flex-start">{icon("lock", 14, C["muted"])}<span>Read-only sheet, schema frozen for V1 (S02).</span></p>'
        f'</div>', bg=C["card"])

    header = top_header(None, nav_active="Catalogue")
    body = (f'<div style="{ROOTSTYLE}; min-height: 620px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">'
            f'<div style="flex: 2 1 640px; min-width: 0">{list_card}</div><div style="flex: 1 1 360px; max-width: 420px">{fiche_card}</div></div></div>')
    return page("Catalogue — computer", body, {"$preview": {"width": 1400, "height": 620}})


# ================= Settings: shared content (computer + phone) =================
def row(ic, title, sub, right_html="", danger=False):
    col = C["crit"] if danger else C["ink"]
    return (f'<div style="display: flex; align-items: center; gap: 14px; padding: 14px 4px; border-top: 1px solid {C["line"]}">'
            f'<span style="flex: none; width: 38px; height: 38px; border-radius: 11px; background: {C["bg"] if not danger else "#FBE7E5"}; display: grid; place-items: center">{icon(ic, 18, col)}</span>'
            f'<div style="flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px"><strong style="font-size: 14.5px; color: {col}">{title}</strong><span style="font-size: 12.5px; color: {C["muted"]}">{sub}</span></div>'
            f'{right_html}</div>')


def reglages_sections(compact=False):
    allee = pill_group("Minimum aisle width", ["30 cm", "50 cm", "80 cm"], 1, "Aisle width")
    export_import = (
        row("doc", "Export the garden", "A file to keep, or to move to another device.",
            secondary_btn("Export", ic="export"))
        + row("upload", "Import a garden", "Replaces the current garden after confirmation.",
              secondary_btn("Import", ic="upload")))
    import_dialog = (
        f'<div style="display: flex; flex-direction: column; gap: 12px; padding: 16px; border-radius: 16px; background: #FDF1E4; border: 1px solid #F3D7AE">'
        f'<div style="display: flex; gap: 10px; align-items: flex-start">{icon("alert", 20, C["warn"])}<strong style="{H}; font-size: 15px">The imported file is older</strong></div>'
        f'<p style="margin: 0; font-size: 13.5px; line-height: 1.5">“garden-export-2026-11-02.json” is dated <strong>2 November 2026</strong>. The garden on this device was changed more recently, on <strong>14 November 2026</strong>.</p>'
        f'<p style="margin: 0; font-size: 13.5px">It will replace <strong>toutes</strong> the current data.</p>'
        f'<div style="display: flex; gap: 10px; justify-content: flex-end">{secondary_btn("Cancel")}{primary_btn("Replace anyway", bg=C["warn"], fg="#3A2808")}</div></div>')
    plan = row("undo", "Clear the plan", "Removes all beds from the current season and the following ones.", secondary_btn("Clear"))
    effacer = row("trash", "Erase the app data", "Deletes the whole garden from this device, after a double confirmation.", secondary_btn("Erase", ic="trash", fg=C["crit"]), danger=True)
    return allee, export_import, import_dialog, plan, effacer


# ================= Artboard: E10 · Settings (computer, panel) =================
def e10_board():
    allee, export_import, import_dialog, plan, effacer = reglages_sections()
    backdrop = (f'<div style="flex: 1 1 820px; min-width: 0; position: relative">'
                f'<div style="background: {C["card"]}; border-radius: 32px; padding: 22px; filter: saturate(0.85); opacity: 0.5">'
                f'<div style="display: flex; align-items: center; gap: 10px; margin-bottom: 14px">{LOGO}<span style="{H}; font-size: 22px">Potagerie</span></div>'
                f'<div style="display: flex; justify-content: center">{flat_plan(cell=30)}</div></div>'
                f'<div aria-hidden="true" style="position: absolute; inset: 0; background: rgba(31,58,46,0.1); border-radius: 32px"></div></div>')
    panel = (
        f'<aside style="flex: 1 1 420px; max-width: 460px; min-width: 0; background: #FFFFFF; border-radius: 28px; padding: 26px; display: flex; flex-direction: column; gap: 18px; box-shadow: 0 20px 48px rgba(31,58,46,0.18)">'
        f'<div style="display: flex; justify-content: space-between; align-items: center"><h1 style="{H}; font-size: 22px">Settings</h1>'
        f'<a href="../../POT-D01_ecrans-cles_v1.1/project/Main.dc.html" aria-label="Close" style="width: 36px; height: 36px; border-radius: 50%; background: {C["bg"]}; display: grid; place-items: center; text-decoration: none">{icon("close", 16)}</a></div>'
        f'{allee}'
        f'<div style="display: flex; flex-direction: column"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}; padding-top: 6px">Data</span>{export_import}</div>'
        f'{import_dialog}'
        f'<div style="display: flex; flex-direction: column">{plan}{effacer}</div>'
        f'</aside>')
    body = (f'<div style="{ROOTSTYLE}; min-height: 820px">'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 28px 32px; align-items: stretch">{backdrop}{panel}</div></div>')
    return page("Settings — computer", body, {"$preview": {"width": 1440, "height": 820}})


# ================= Artboard: E10 · Settings (phone, full screen) =================
def e10_mobile_board():
    allee, export_import, import_dialog, plan, effacer = reglages_sections(compact=True)
    header = (f'<div style="display: flex; align-items: center; gap: 12px; padding: 16px 18px 10px">'
              f'<a href="../../POT-D01_ecrans-cles_v1.1/project/E05_garden_mobile.dc.html" aria-label="Back" style="width: 38px; height: 38px; border-radius: 50%; border: 1px solid {C["line"]}; background: #FFFFFF; display: grid; place-items: center; text-decoration: none">{icon("back", 16)}</a>'
              f'<h1 style="{H}; font-size: 19px">Settings</h1></div>')
    body = (
        f'<div style="{ROOTSTYLE}; width: 390px; height: 844px; box-sizing: border-box; display: flex; flex-direction: column; overflow-y: auto">'
        f'<span aria-hidden="true" style="align-self: center; width: 120px; height: 28px; margin-top: 4px; flex: none"></span>'
        f'{header}'
        f'<div style="flex: 1; padding: 0 18px 26px; display: flex; flex-direction: column; gap: 16px">'
        f'{allee}'
        f'<div><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Data</span>{export_import}</div>'
        f'{import_dialog}'
        f'<div>{plan}{effacer}</div>'
        f'</div>'
        f'<div aria-hidden="true" style="align-self: center; width: 134px; height: 5px; border-radius: 99px; background: {C["ink"]}; opacity: 0.25; margin: 6px 0 10px"></div>'
        f'</div>')
    return page("Settings — phone", body, {"$preview": {"width": 390, "height": 844}})


# ================= Artboard: E11 · Timeline export (computer) =================
YEAR = 2027
MONTHS = ["Jan.", "Feb.", "Mar", "Apr.", "May", "Jun", "Jul.", "Aug", "Sept.", "Oct.", "Nov.", "Dec."]
MDAYS = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]


def doy(md):
    m, d = int(md[:2]), int(md[3:])
    return sum(MDAYS[:m - 1]) + d - 1


def pct(md):
    return doy(md) / 365 * 100


EXPORT_SERIES = [
    # (bed, name, crop, label, planting out, start of harvest, end [clipped at 31 Dec if straddling the next year])
    ("A", "Back of the garden", "carotte", "Carrot", "03-15", "06-23", "08-02"),
    ("B", "The tomatoes", "tomate", "Tomato", "05-05", "07-19", "09-27"),
    ("B", "The tomatoes", "laitue", "Lettuce", "05-13", "07-02", "07-22"),
    ("C", "The zucchinis", "courgette", "Zucchini", "05-01", "06-20", "09-08"),
    ("D", "Roots", "carotte", "Carrot", "03-15", "06-23", "08-02"),
    ("D", "Roots", "poireau", "Leek", "07-10", "11-07", "12-31"),
    ("E", "Shed side", "haricot", "Green bean", "06-10", "08-09", "09-08"),
    ("F", "Leek square", "poireau", "Leek", "05-19", "09-16", "12-15"),
    ("G", "Salad square", "laitue", "Lettuce", "04-20", "06-09", "06-29"),
    ("H", "End of the aisle", "laitue", "Lettuce", "08-02", "09-21", "10-06"),
]


def export_gantt_preview():
    cols = " ".join(f"{d}fr" for d in MDAYS)
    months = "".join(f'<div style="font-size: 10px; font-weight: 600; letter-spacing: 0.03em; text-transform: uppercase; color: {C["muted"]}; padding: 0 0 6px 4px; border-left: 1px solid {C["line"]}">{m[:4]}</div>' for m in MONTHS)
    rows_by_planche = {}
    for pid, nom, crop, label, deb, rec, fin in EXPORT_SERIES:
        rows_by_planche.setdefault((pid, nom), []).append((crop, label, deb, rec, fin))
    labels, tracks = "", ""
    for (pid, nom), items in rows_by_planche.items():
        # one sub-row per planting when they overlap (F-VIS-2), planter included
        n = len(items)
        row_h = 30 if n <= 1 else 18 * n + 6
        lane_h = 20 if n <= 1 else 14
        labels += f'<div style="height: {row_h}px; display: flex; align-items: center; padding: 0 8px; font-size: 12px; font-weight: 600; border-top: 1px solid {C["line"]}">{nom}</div>'
        bars = ""
        for i, (crop, label, deb, rec, fin) in enumerate(items):
            left, right = pct(deb), pct(fin) + 1 / 365 * 100
            dG, dH = doy(rec) - doy(deb), doy(fin) - doy(rec)
            col = CROPCOL[crop]
            segs = f'<div style="flex: {dG} 1 0; background: {mix(col, 0.45)}"></div><div style="flex: {dH if dH > 0 else 1} 1 0; background: {col}"></div>'
            top = 4 if n <= 1 else 4 + i * (lane_h + 4)
            bars += (f'<div style="position: absolute; left: {left:.2f}%; width: {right - left:.2f}%; top: {top}px; height: {lane_h}px; border-radius: 6px; overflow: hidden; display: flex">{segs}</div>')
        tracks += f'<div style="position: relative; height: {row_h}px; border-top: 1px solid {C["line"]}">{bars}</div>'
    return (f'<div style="display: grid; grid-template-columns: 120px minmax(0, 1fr)">'
            f'<div></div><div style="display: grid; grid-template-columns: {cols}">{months}</div>'
            f'<div style="display: flex; flex-direction: column; border-bottom: 1px solid {C["line"]}">{labels}</div>'
            f'<div style="position: relative; border-bottom: 1px solid {C["line"]}">{tracks}</div></div>')


def e11_board():
    fmt_opts = "".join(
        f'<button aria-pressed="{"true" if sel else "false"}" style="flex: 1; display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 14px; border-radius: 16px; cursor: pointer; '
        f'border: {"2px solid " + C["accent"] if sel else "2px solid " + C["line"]}; background: {"#FFF3E4" if sel else "#FFFFFF"}">{icon(ic, 24, C["ink"])}<strong style="font-size: 13.5px">{t}</strong><span style="font-size: 11.5px; color: {C["muted"]}">{sub}</span></button>'
        for ic, t, sub, sel in (("doc", "PDF", "A4 or A3 landscape, to print", True), ("image", "Image PNG", "To share or keep a record", False)))

    periode = (f'<div style="display: flex; gap: 10px">{field("exp-d", "From", "1 Jan 2027")}{field("exp-f", "Au", "31 Dec 2027")}</div>'
               f'<p style="margin: -4px 0 0; font-size: 12.5px; color: {C["muted"]}">By default, the period shown on screen.</p>')

    planches_chips = "".join(
        f'<label style="display: inline-flex; align-items: center; gap: 7px; padding: 7px 12px; border-radius: 99px; background: {C["bg"]}; font-size: 13px; font-weight: 600">'
        f'<span aria-hidden="true" style="width: 15px; height: 15px; border-radius: 4px; background: {C["green"]}"></span>{nom}</label>'
        for _, nom in sorted({(pid, nom) for pid, nom, *_ in EXPORT_SERIES}))
    planches_sel = (f'<div style="display: flex; justify-content: space-between; align-items: baseline"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">Beds</span>'
                    f'<a href="#" style="font-size: 12.5px; font-weight: 600">Select all</a></div><div style="display: flex; flex-wrap: wrap; gap: 8px">{planches_chips}</div>')

    toggles = "".join(
        f'<label style="display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 10px 2px; border-top: 1px solid {C["line"]}">'
        f'<span style="font-size: 14px">{t}</span><span style="width: 36px; height: 21px; border-radius: 99px; background: {C["good"] if on else C["line"]}; position: relative; flex: none">'
        f'<span style="position: absolute; top: 2px; {"right" if on else "left"}: 2px; width: 17px; height: 17px; border-radius: 50%; background: #FFFFFF"></span></span></label>'
        for t, on in (("Phases (under cover, growth, harvest)", True), ("“Today” marker", True), ("Annotations", True), ("Task calendar", True)))

    form = card(
        f'<div style="display: flex; flex-direction: column; gap: 20px">'
        f'<h1 style="{H}; font-size: 24px">Export the timeline</h1>'
        f'<div style="display: flex; gap: 10px">{fmt_opts}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 8px"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">Period</span>{periode}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 8px">{planches_sel}</div>'
        f'<div style="display: flex; flex-direction: column"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}; padding-top: 6px">Content</span>{toggles}</div>'
        f'{primary_btn("Exporter en PDF", ic="export", block=True)}'
        f'</div>')

    legend = "".join(f'<span style="display: inline-flex; align-items: center; gap: 7px; font-size: 12.5px; color: {C["muted"]}"><span style="width: 22px; height: 11px; border-radius: 4px; {bg}"></span>{t}</span>'
                      for bg, t in ((f"background: {mix('#E5533D', 0.45)}", "Growth"), ("background: #E5533D", "Harvest")))
    preview = card(
        f'<div style="display: flex; flex-direction: column; gap: 14px">'
        f'<div style="display: flex; justify-content: space-between; align-items: baseline"><strong style="font-size: 14px">Preview</strong><span style="font-size: 12.5px; color: {C["muted"]}">Export of 14 Nov 2026</span></div>'
        f'<div style="padding: 16px; border-radius: 18px; background: #FFFFFF">'
        f'<div style="display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 10px"><strong style="{H}; font-size: 16px">Tim’s garden</strong><span style="font-size: 12px; color: {C["muted"]}">Year 2027</span></div>'
        f'<div style="display: flex; gap: 16px; margin-bottom: 14px">'
        f'<div style="flex: none">{flat_plan(cell=11, labels=False)}</div>'
        f'<div style="flex: 1; min-width: 0; align-self: center">{export_gantt_preview()}</div></div>'
        f'<div style="display: flex; gap: 14px; flex-wrap: wrap; padding-top: 10px; border-top: 1px solid {C["line"]}">{legend}</div></div>'
        f'<p style="margin: 0; font-size: 12.5px; color: {C["muted"]}">The PDF adds a thumbnail of the plan and, if ticked, the month-by-month list of tasks to do.</p>'
        f'</div>', bg=C["card"])

    header = top_header(None, nav_active=None)
    body = (f'<div style="{ROOTSTYLE}; min-height: 600px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">'
            f'<div style="flex: 1 1 400px; max-width: 440px">{form}</div><div style="flex: 2 1 640px; min-width: 0">{preview}</div></div></div>')
    return page("Timeline export — computer", body, {"$preview": {"width": 1440, "height": 600}})


# ================= Artboard : ET01 · Cross-cutting states (ordinateur) =================
def state_card(title, body_html, height=360):
    return (f'<div style="display: flex; flex-direction: column; gap: 12px; background: #FFFFFF; border-radius: 24px; padding: 22px; height: {height}px; box-sizing: border-box">'
            f'<strong style="{H}; font-size: 16px">{title}</strong><div style="flex: 1; min-width: 0; display: flex; flex-direction: column">{body_html}</div></div>')


def et01_board():
    # 1. First launch, no plot
    s1 = (f'<div style="flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 14px; text-align: center; background: {C["card"]}; border-radius: 18px; padding: 20px">'
          f'{LOGO}<p style="margin: 0; font-size: 14px; color: {C["muted"]}; max-width: 320px">No plot yet: nothing is drawn yet.</p>'
          f'{primary_btn("Create my plot", ic="plus")}</div>')

    # 2. Empty timeline
    cols = " ".join("1fr" for _ in MONTHS)
    months2 = "".join(f'<div style="font-size: 10px; font-weight: 600; text-transform: uppercase; color: {C["line"]}; border-left: 1px solid {C["line"]}; padding-left: 4px">{m[:4]}</div>' for m in MONTHS)
    s2 = (f'<div style="flex: 1; display: flex; flex-direction: column; gap: 10px">'
          f'<div style="display: grid; grid-template-columns: {cols}">{months2}</div>'
          f'<div style="flex: 1; border: 1px dashed {C["line"]}; border-radius: 14px; display: flex; align-items: center; justify-content: center; text-align: center; padding: 16px">'
          f'<p style="margin: 0; font-size: 13.5px; color: {C["muted"]}; max-width: 280px">No planting yet.<br>Add a bed in the editor, then plant it.</p></div></div>')

    # 3. Move refused (conflict, S03 T12)
    s3 = (f'<div style="flex: 1; display: flex; flex-direction: column; gap: 10px">'
          f'<div style="display: flex; justify-content: center">{flat_plan(cell=22, planches=[p for p in PLANCHES if p["id"] in ("B", "E")], gw=18, gh=6, highlight="B", drag_to=("E", 1, 3), labels=True)}</div>'
          f'<div style="display: flex; gap: 10px; align-items: flex-start; padding: 12px 14px; border-radius: 14px; background: #FBE7E5">{icon("alert", 18, C["crit"])}'
          f'<p style="margin: 0; font-size: 13px; line-height: 1.4; color: {C["accentInk"]}"><strong>Position refused.</strong> “Shed side” would overlap “The tomatoes”. The bed goes back to its place.</p></div></div>')

    # 4. File write error
    s4 = (f'<div style="flex: 1; display: flex; flex-direction: column; gap: 14px; align-items: center; justify-content: center; text-align: center; background: {C["card"]}; border-radius: 18px; padding: 20px">'
          f'<span style="display: inline-flex; align-items: center; gap: 8px; padding: 8px 16px; border-radius: 99px; background: #FBE7E5; color: {C["crit"]}; font-size: 13.5px; font-weight: 700">{icon("alert", 16, C["crit"])}Erreur d\'enregistrement</span>'
          f'<p style="margin: 0; font-size: 13.5px; color: {C["muted"]}; max-width: 300px">The file could not be saved (disk full or folder unavailable). Nothing is lost: the change stays displayed.</p>'
          f'{secondary_btn("Retry", ic="undo")}</div>')

    grid = (f'<div style="display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 20px">'
            f'{state_card("① First launch, no plot", s1)}{state_card("② Empty timeline", s2)}'
            f'{state_card("③ Move refused (conflict, S03 T12)", s3)}{state_card("④ File write error", s4)}</div>')

    body = (f'<div style="{ROOTSTYLE}; width: 1360px; box-sizing: border-box; padding: 40px">'
            f'<div style="display: flex; flex-direction: column; gap: 6px; margin-bottom: 24px"><span style="font-size: 13px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: {C["muted"]}">Cross-cutting states</span>'
            f'<h1 style="{H}; font-size: 30px">Four states that can occur on any screen</h1></div>{grid}</div>')
    return page("Cross-cutting states — computer", body, {"$preview": {"width": 1360, "height": 920}})


BOARDS = {
    "Main.dc.html": e02_board,
    "E06_bed-panel_desktop.dc.html": e06_board,
    "E07_planting_desktop.dc.html": e07_board,
    "E09_catalogue_desktop.dc.html": e09_board,
    "E10_settings_desktop.dc.html": e10_board,
    "E10_settings_mobile.dc.html": e10_mobile_board,
    "E11_timeline-export_desktop.dc.html": e11_board,
    "ET01_states_desktop.dc.html": et01_board,
}

if __name__ == "__main__":
    for name, fn in BOARDS.items():
        with open(os.path.join(ROOT, name), "w", encoding="utf-8") as f:
            f.write(fn())
    canvas = {
        "v": 3,
        "createdOnFiles": {"v": 1, "at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")},
        "title": "POT-D02 · Additional screens V1",
        "launch": {"view": "canvas"},
        "pages": [],
        "boards": {
            "Main.dc.html": {"x": 0, "y": 0, "w": 1320, "h": 900, "title": "① E02 · Plot creation — computer", "expand": "fill", "is_interactive": True},
            "E06_bed-panel_desktop.dc.html": {"x": 1480, "y": 0, "w": 1440, "h": 760, "title": "④ E06 · Bed panel — computer", "expand": "fill", "is_interactive": True},
            "E07_planting_desktop.dc.html": {"x": 3080, "y": 0, "w": 1320, "h": 720, "title": "⑤ E07 · New planting — computer", "expand": "fill", "is_interactive": True},
            "E09_catalogue_desktop.dc.html": {"x": 0, "y": 1000, "w": 1400, "h": 620, "title": "E09 · Catalogue — computer", "expand": "fill", "is_interactive": True},
            "E10_settings_desktop.dc.html": {"x": 1560, "y": 1000, "w": 1440, "h": 820, "title": "E10 · Settings — computer", "expand": "fill", "is_interactive": True},
            "E10_settings_mobile.dc.html": {"x": 3160, "y": 1000, "w": 390, "h": 844, "title": "E10 · Settings — phone", "is_interactive": True},
            "E11_timeline-export_desktop.dc.html": {"x": 0, "y": 1780, "w": 1440, "h": 600, "title": "E11 · Timeline export — computer", "expand": "fill", "is_interactive": True},
            "ET01_states_desktop.dc.html": {"x": 1600, "y": 1780, "w": 1360, "h": 920, "title": "ET01 · Cross-cutting states — computer"},
        },
        "order": ["Main.dc.html", "E06_bed-panel_desktop.dc.html", "E07_planting_desktop.dc.html", "E09_catalogue_desktop.dc.html",
                  "E10_settings_desktop.dc.html", "E10_settings_mobile.dc.html", "E11_timeline-export_desktop.dc.html", "ET01_states_desktop.dc.html"],
        "notes": {
            "titre": {"x": 0, "y": -300, "text": "Flow 6.1: from plot creation to planning (continued in D01)", "kind": "title1", "maxW": 4400},
            "lien1": {"x": 1340, "y": 40, "w": 160, "fill": "green", "size": "s",
                      "text": "① → D01 E03\n“Create the plot” opens the plan editor (D01)."},
            "lien2": {"x": 2940, "y": 40, "w": 160, "fill": "green", "size": "s",
                      "text": "D01 E05 → ④\n“Edit” in the simplified panel of D01 opens the full panel.\n\n④ → ⑤\n“Plant here”."},
            "lien3": {"x": 4540, "y": 40, "w": 160, "fill": "green", "size": "s",
                      "text": "⑤ → D01 E05\n“Plant …” goes back to the garden.\n\n④ → D01 E03\n“Move”."},
            "titre2": {"x": 0, "y": 900, "text": "Side screens (catalogue, settings, export) — settings on a phone too", "kind": "title1", "maxW": 4000},
            "titre3": {"x": 0, "y": 1680, "text": "Cross-cutting states", "kind": "title1", "maxW": 1360},
            "hyp": {"x": 1600, "y": 2760, "w": 360, "fill": "orange",
                    "text": "Example garden: 8 beds out of 10, the 6 reference crops (S02), 2027 season.\nDates and quantities checked by an S03 script (see the README), not computed by hand.\nThe Play button of each sheet follows the links, including to D01."},
        },
        "designSystems": [],
    }
    with open(os.path.join(ROOT, "canvas.json"), "w", encoding="utf-8") as f:
        json.dump(canvas, f, ensure_ascii=False, indent=1)
    print("ok", sorted(os.listdir(ROOT)))
