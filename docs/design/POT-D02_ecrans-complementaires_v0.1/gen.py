#!/usr/bin/env python3
"""Génère les artboards .dc.html du canvas Potagerie — écrans complémentaires V1 (D02).
Dates et quantités vérifiées par un script à part (S03 v0.5) ; voir le README.
Les dessins de légumes viennent de plants2.py, copié depuis D01 (voir le README)."""
import json, os, sys, datetime

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "project")
os.makedirs(ROOT, exist_ok=True)
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from plants2 import plant  # noqa: E402

# ---------- Palette (identique à D01) ----------
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
    items = [("settings", "Réglages", "E10_reglages_desktop.dc.html"), ("book", "Catalogue", "E09_catalogue_desktop.dc.html")]
    out = ""
    for ic, label, href in items:
        on = label == active
        out += (f'<a href="{href}" aria-label="{label}" style="display: inline-flex; align-items: center; justify-content: center; width: 40px; height: 40px; border-radius: 50%; '
                f'background: {"#FFFFFF" if not on else C["ink"]}; border: 1px solid {C["line"]}; text-decoration: none">{icon(ic, 17, C["ink"] if not on else "#FFFFFF")}</a>')
    return out


def top_header(title_small, subtitle=None, nav_active=None, back_href=None, close_href=None):
    left = (f'<a href="{back_href}" aria-label="Retour" style="width: 40px; height: 40px; border-radius: 50%; border: 1px solid {C["line"]}; background: #FFFFFF; display: grid; place-items: center; text-decoration: none">{icon("back", 17)}</a>'
            if back_href else f'<div style="display: flex; align-items: center; gap: 10px">{LOGO}<span style="{H}; font-size: 22px; color: {C["ink"]}">Potagerie</span></div>')
    titleblk = (f'<div style="display: flex; flex-direction: column; gap: 1px"><strong style="{H}; font-size: 17px">{title_small}</strong>'
                f'{"<span style=" + chr(34) + "font-size: 12.5px; color: " + C["muted"] + chr(34) + ">" + subtitle + "</span>" if subtitle else ""}</div>') if title_small else ""
    right = nav_icons(nav_active)
    if close_href:
        right += f'<a href="{close_href}" aria-label="Fermer" style="width: 40px; height: 40px; border-radius: 50%; background: {C["bg"]}; display: grid; place-items: center; text-decoration: none">{icon("close", 16)}</a>'
    return (f'<header style="display: flex; align-items: center; gap: 14px; padding: 18px 28px">{left}{titleblk}<div style="flex: 1"></div>{right}</header>')


# ---------- Plan plat (vue de dessus, non isométrique) ----------
GW, GH = 18, 13  # 9 x 6,5 m, case 50 cm

PLANCHES = [
    dict(id="A", x=0, y=0, w=6, h=2, kind="bed", nom="Fond du jardin"),
    dict(id="D", x=7, y=0, w=6, h=2, kind="bed", nom="Racines"),
    dict(id="B", x=0, y=3, w=8, h=2, kind="bed", nom="Les tomates"),
    dict(id="E", x=9, y=3, w=8, h=2, kind="bed", nom="Côté cabane"),
    dict(id="C", x=0, y=6, w=8, h=2, kind="bed", nom="Les courgettes"),
    dict(id="F", x=9, y=6, w=4, h=4, kind="bac", nom="Carré poireaux"),
    dict(id="G", x=14, y=6, w=3, h=4, kind="bac", nom="Carré salades"),
    dict(id="H", x=0, y=9, w=6, h=2, kind="butte", nom="Au bout de l'allée"),
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
    return f'<svg viewBox="0 0 {W} {Hh}" width="100%" style="display: block; max-width: {W}px" role="img" aria-label="Plan du potager vu de dessus">{body}</svg>'


# ---------- barre de phases (frise plate, réutilisée pour E06/E07/E11) ----------
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


# ---------- composants de formulaire ----------
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


# ================= Artboard : E02 · Création du terrain (ordinateur, Main.dc.html) =================
def e02_board():
    steps = "".join(
        f'<span style="display: inline-flex; align-items: center; gap: 6px; font-size: 13px; font-weight: 600; color: {C["ink"] if on else C["muted"]}">'
        f'<span style="width: 22px; height: 22px; border-radius: 50%; display: grid; place-items: center; font-size: 12px; font-weight: 700; '
        f'background: {C["ink"] if on else "#FFFFFF"}; color: {"#FFFFFF" if on else C["muted"]}; border: 1px solid {C["line"]}">{n}</span>{t}</span>'
        for n, t, on in ((1, "Terrain", True), (2, "Planches", False)))
    header = (f'<header style="display: flex; align-items: center; gap: 18px; padding: 20px 32px">'
              f'<div style="display: flex; align-items: center; gap: 10px">{LOGO}<span style="{H}; font-size: 24px">Potagerie</span></div>'
              f'<div style="flex: 1"></div><div style="display: flex; gap: 20px">{steps}</div></header>')

    form = card(
        f'<div style="display: flex; flex-direction: column; gap: 22px">'
        f'<div style="display: flex; flex-direction: column; gap: 4px"><h1 style="{H}; font-size: 26px">Créer son terrain</h1>'
        f'<p style="margin: 0; font-size: 14px; color: {C["muted"]}">La surface totale, rectangulaire, qu\'on découpera ensuite en planches dans l\'atelier.</p></div>'
        f'<div style="display: flex; gap: 14px">{field("t-l", "Longueur", "9,0", "m")}{field("t-w", "Largeur", "6,5", "m")}</div>'
        f'{pill_group("Taille de case", ["25 cm", "50 cm", "100 cm"], 1, "Taille de case")}'
        f'<p style="margin: -8px 0 0; display: flex; gap: 8px; font-size: 13px; color: {C["muted"]}; align-items: flex-start">{icon("lock", 15, C["muted"])}'
        f'<span>La taille de case fixe la taille des tronçons des planches : elle ne pourra plus être changée une fois le terrain créé.</span></p>'
        f'{pill_group("Largeur d\'allée minimale", ["30 cm", "50 cm", "80 cm"], 1, "Largeur d\'allée")}'
        f'<p style="margin: -8px 0 0; font-size: 13px; color: {C["muted"]}">Gardée entre les planches dans l\'atelier. Modifiable à tout moment dans les réglages.</p>'
        f'<div style="display: flex; gap: 12px; padding-top: 6px">{primary_btn("Créer le terrain", "../../POT-D01_ecrans-cles_v1.1/project/E03_atelier-plan_desktop.dc.html", "check")}</div>'
        f'</div>')

    preview = card(
        f'<div style="display: flex; flex-direction: column; gap: 14px">'
        f'<div style="display: flex; justify-content: space-between; align-items: baseline"><strong style="font-size: 14px">Aperçu de la grille</strong>'
        f'<span style="font-size: 13px; color: {C["muted"]}">9,0 × 6,5 m · case 50 cm · 18 × 13 cases</span></div>'
        f'<div style="display: flex; justify-content: center">{flat_plan(cell=26, planches=[], labels=False)}</div>'
        f'<p style="margin: 0; font-size: 13px; color: {C["muted"]}">Le terrain est encore vide : on y glissera les premières planches dans l\'atelier du plan.</p>'
        f'</div>', bg=C["card"])

    body = (f'<div style="{ROOTSTYLE}; min-height: 900px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">'
            f'<div style="flex: 1 1 460px; max-width: 520px">{form}</div>'
            f'<div style="flex: 1 1 460px; max-width: 560px">{preview}</div></div></div>')
    return page("Création du terrain — ordinateur", body, {"$preview": {"width": 1320, "height": 900}})


# ================= Artboard : E06 · Panneau planche (ordinateur) =================
def series_row(kind, nom, qty_txt, dates_txt, stage_txt, stage_bg=None):
    return (f'<div style="display: flex; gap: 12px; align-items: flex-start; padding: 12px; border-radius: 18px; background: {C["bg"]}">'
            f'<span style="flex: none; width: 48px; height: 48px; border-radius: 13px; background: #FFFFFF; display: grid; place-items: center">{crop_icon(kind, 2, 40)}</span>'
            f'<div style="display: flex; flex-direction: column; gap: 3px; min-width: 0"><div style="display: flex; align-items: center; gap: 8px"><strong style="font-size: 15px">{nom}</strong>'
            f'<span style="font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 99px; background: {stage_bg or "#FFFFFF"}; color: {C["green"] if not stage_bg else C["accentInk"]}">{stage_txt}</span></div>'
            f'<span style="font-size: 13px; color: {C["muted"]}">{qty_txt}</span><span style="font-size: 13px">{dates_txt}</span></div></div>')


def e06_board():
    # dimmed backdrop (écran Potager, planche Racines en surbrillance)
    backdrop = (
        f'<div style="position: relative; flex: 1 1 820px; min-width: 0; background: {C["card"]}; border-radius: 32px; padding: 22px; filter: saturate(0.85); opacity: 0.55">'
        f'<div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px">'
        f'<div style="display: flex; align-items: center; gap: 10px">{LOGO}<span style="{H}; font-size: 22px">Potagerie</span></div>'
        f'<span style="{H}; font-size: 20px">20 juin 2027</span></div>'
        f'<div style="display: flex; justify-content: center">{flat_plan(cell=30, highlight="D")}</div>'
        f'</div>'
        f'<div aria-hidden="true" style="position: absolute; inset: 0; background: rgba(31,58,46,0.12); border-radius: 32px"></div>')

    hist = (f'<details open style="display: flex; flex-direction: column; gap: 8px">'
            f'<summary style="cursor: pointer; list-style: none; display: flex; justify-content: space-between; align-items: center; font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">'
            f'<span style="display: inline-flex; align-items: center; gap: 6px">{icon("lock", 13, C["muted"])}Historique · lecture seule</span>{icon("down", 14, C["muted"])}</summary>'
            f'<div style="display: flex; gap: 12px; align-items: flex-start; padding: 12px; border-radius: 18px; background: {C["bg"]}; opacity: 0.85">'
            f'<span style="flex: none; width: 48px; height: 48px; border-radius: 13px; background: #FFFFFF; display: grid; place-items: center">{crop_icon("carotte", 3, 40)}</span>'
            f'<div style="display: flex; flex-direction: column; gap: 3px"><strong style="font-size: 14px">2026 · Carotte</strong>'
            f'<span style="font-size: 13px; color: {C["muted"]}">15 mars → 2 août · ≈ 148 pieds (4 rangs × 37)</span></div></div>'
            f'</details>')

    actions = (
        f'<div style="display: flex; gap: 10px; flex-wrap: wrap; margin-top: auto">'
        f'<a href="E07_serie_desktop.dc.html" style="flex: 1 1 130px; display: inline-flex; justify-content: center; align-items: center; gap: 8px; background: {C["green"]}; color: #FFFFFF; text-decoration: none; border-radius: 99px; padding: 12px 14px; font: 600 14.5px Figtree, sans-serif">{icon("plus", 17, "#FFFFFF")}Planter ici</a>'
        f'<a href="../../POT-D01_ecrans-cles_v1.1/project/E03_atelier-plan_desktop.dc.html" style="flex: 1 1 110px; display: inline-flex; justify-content: center; align-items: center; gap: 8px; border: 1px solid {C["line"]}; background: #FFFFFF; color: {C["ink"]}; text-decoration: none; border-radius: 99px; padding: 12px 14px; font: 600 14.5px Figtree, sans-serif">{icon("move", 16)}Déplacer</a>'
        f'<button style="flex: 1 1 110px; border: 1px solid {C["line"]}; background: #FFFFFF; color: {C["crit"]}; border-radius: 99px; padding: 12px 14px; font: 600 14.5px Figtree, sans-serif; cursor: pointer; display: inline-flex; justify-content: center; align-items: center; gap: 8px">{icon("trash", 16, C["crit"])}Supprimer</button>'
        f'</div>')

    panel = (
        f'<aside style="flex: 1 1 400px; max-width: 440px; min-width: 0; background: #FFFFFF; border-radius: 28px; padding: 26px; display: flex; flex-direction: column; gap: 18px; box-shadow: 0 20px 48px rgba(31,58,46,0.18)">'
        f'<div style="display: flex; justify-content: space-between; align-items: center; gap: 8px">'
        f'<label for="pl-name" style="flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Planche D</span>'
        f'<input id="pl-name" value="Racines" style="{H}; font-size: 24px; color: {C["ink"]}; border: 0; border-bottom: 2px dashed {C["line"]}; background: transparent; padding: 2px 0; width: 100%"></label>'
        f'<a href="../../POT-D01_ecrans-cles_v1.1/project/Main.dc.html" aria-label="Fermer le panneau" style="width: 36px; height: 36px; border-radius: 50%; background: {C["bg"]}; display: grid; place-items: center; text-decoration: none">{icon("close", 16)}</a></div>'
        f'<div style="display: flex; gap: 8px">{chip("3 × 1,2 m", C["bg"])}{chip("3,6 m²", C["bg"])}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 10px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Séries de 2027</span>'
        f'{series_row("carotte", "Carotte", "Tronçons 1–3 · ≈ 148 pieds (4 rangs × 37)", "15 mars → récolte dès le 23 juin", "Adulte")}'
        f'{series_row("poireau", "Poireau", "Tronçons 4–6 · ≈ 48 pieds (4 rangs × 12)", "Semé à l\'abri le 1er mai · repiquage le 10 juillet", "Semé à l\'abri", "#FFF3E4")}'
        f'</div>'
        f'{hist}'
        f'<div style="display: flex; flex-direction: column; gap: 6px"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Avant sur cette planche</span>'
        f'<div style="display: flex; justify-content: space-between; font-size: 14px"><span>2026</span><span style="color: {C["muted"]}">Apiacées (carotte)</span></div>'
        f'<div style="display: flex; justify-content: space-between; font-size: 14px"><span>2025</span><span style="color: {C["muted"]}">Rien de noté</span></div></div>'
        f'{actions}'
        f'</aside>')

    confirm = (
        f'<div style="position: absolute; inset: 0; background: rgba(31,58,46,0.45); display: flex; align-items: center; justify-content: center; border-radius: 32px">'
        f'<div style="width: 420px; background: #FFFFFF; border-radius: 24px; padding: 26px; display: flex; flex-direction: column; gap: 14px; box-shadow: 0 24px 60px rgba(31,58,46,0.3)">'
        f'<div style="display: flex; gap: 10px; align-items: flex-start">{icon("alert", 22, C["warn"])}<h2 style="{H}; font-size: 19px">Supprimer Racines ?</h2></div>'
        f'<p style="margin: 0; font-size: 14px; line-height: 1.5">Si tu continues, la planche sera retirée du plan à partir de <strong>2028</strong>.</p>'
        f'<div style="padding: 12px 14px; border-radius: 14px; background: #FDF1E4; font-size: 13.5px; line-height: 1.5">'
        f'<strong>1 série</strong> sera supprimée, car elle touche 2028 — y compris à cheval sur l\'année : <strong>Poireau</strong> (10 juil. 2027 → 5 fév. 2028).<br>'
        f'La <strong>Carotte</strong>, déjà terminée le 2 août 2027, est gardée.</div>'
        f'<div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 4px">{secondary_btn("Annuler")}{primary_btn("Supprimer la planche", ic="trash", bg=C["crit"])}</div>'
        f'</div></div>')

    stage = (f'<div style="flex: 1 1 820px; min-width: 0; position: relative">{backdrop}{confirm}</div>')
    body = (f'<div style="{ROOTSTYLE}; min-height: 760px">'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 28px 32px; align-items: stretch">{stage}{panel}</div>'
            f'</div>')
    return page("Panneau planche — ordinateur", body, {"$preview": {"width": 1440, "height": 760}})


# ================= Artboard : E07 · Nouvelle série (ordinateur) =================
def e07_board():
    lab = lambda n, t: (f'<div style="display: flex; align-items: center; gap: 10px"><span style="width: 24px; height: 24px; border-radius: 50%; background: {C["ink"]}; color: #FFFFFF; display: grid; place-items: center; font: 700 13px Figtree, sans-serif">{n}</span>'
                        f'<span style="font: 600 17px Fredoka, Figtree, sans-serif; color: {C["ink"]}">{t}</span></div>')
    crops = [("poireau", "Poireau", False), ("laitue", "Laitue", True), ("carotte", "Carotte", False),
             ("haricot", "Haricot vert", False), ("courgette", "Courgette", False), ("tomate", "Tomate", False)]
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
    modes = [(ic_godet, "Je sème à l'abri", "En godets, puis repiquage au jardin", False),
             (ic_plant, "J'achète des plants", "Prêts à planter, en jardinerie", True),
             (ic_seed, "Je sème en pleine terre", "Directement dans la planche", False)]
    mode_cards = "".join(
        f'<button aria-pressed="{"true" if sel else "false"}" style="display: flex; align-items: center; gap: 12px; width: 100%; text-align: left; padding: 11px 14px; border-radius: 16px; cursor: pointer; '
        f'border: {"2px solid " + C["accent"] if sel else "2px solid " + C["line"]}; background: {"#FFF3E4" if sel else "#FFFFFF"}; font: 600 15px Figtree, sans-serif; color: {C["ink"]}">'
        f'<span style="flex: none; width: 46px; height: 46px; border-radius: 12px; background: {"#FFFFFF" if sel else C["bg"]}; display: grid; place-items: center">{ic}</span>'
        f'<span style="display: flex; flex-direction: column; gap: 1px; flex: 1"><span>{t}</span><span style="font-size: 12px; font-weight: 500; color: {C["muted"]}">{sub}</span></span>'
        f'<span aria-hidden="true" style="flex: none; width: 20px; height: 20px; border-radius: 50%; border: 2px solid {C["accent"] if sel else C["line"]}; box-sizing: border-box; display: grid; place-items: center">{"<span style=" + chr(34) + "width: 10px; height: 10px; border-radius: 50%; background: " + C["accent"] + chr(34) + "></span>" if sel else ""}</span></button>'
        for ic, t, sub, sel in modes)

    # mini-frise : 2 août (plantation) -> 21 sept (récolte) -> 6 oct (fin raccourcie, au lieu du 11 oct)
    bar = phase_bar(0, 50, 15, CROPCOL["laitue"], height=16)
    keys = [("2 août", "Plantation", "flex-start"), ("21 sept.", "1re récolte", "center"), ("6 oct.", "Fin modifiée", "flex-end")]
    marks = "".join(
        f'<div style="display: flex; flex-direction: column; align-items: {al}; white-space: nowrap">'
        f'<strong style="font-size: 13px">{d}</strong><span style="font-size: 11px; color: {C["muted"] if t != "Fin modifiée" else C["accentInk"]}; font-weight: {700 if t == "Fin modifiée" else 400}">{t}</span></div>'
        for d, t, al in keys)
    manual_badge = (f'<span style="display: inline-flex; align-items: center; gap: 6px; padding: 3px 10px; border-radius: 99px; background: #FDF1E4; color: {C["accentInk"]}; font-size: 12px; font-weight: 700">'
                    f'{icon("edit", 13, C["accentInk"])}Modifiée à la main : fin raccourcie au 6 oct.</span>'
                    f'<a href="#" style="font-size: 12px; font-weight: 600">Recalculer (11 oct.)</a>')
    timeline = (f'<div style="display: flex; flex-direction: column; gap: 10px; padding: 16px; border-radius: 18px; background: {C["bg"]}">'
                f'{bar}<div style="display: flex; justify-content: space-between">{marks}</div>'
                f'<div style="display: flex; flex-wrap: wrap; gap: 8px; align-items: center; padding-top: 10px; border-top: 1px solid {C["line"]}">'
                f'<span style="font-size: 13px; color: {C["muted"]}">Créneau libre · 2 août → 5 oct. 2027 (65 j)</span>{manual_badge}</div></div>')

    parts = "".join(
        f'<button aria-pressed="{"true" if on else "false"}" style="flex: 1; border: 0; padding: 8px 0; border-radius: 99px; font: 600 13px Figtree, sans-serif; cursor: pointer; '
        f'{"background: " + C["ink"] + "; color: #FFFFFF" if on else "background: transparent; color: " + C["muted"]}">{t}</button>'
        for t, on in (("Toute", True), ("½", False), ("⅓", False), ("¼", False)))
    tsegs = "".join(
        f'<button aria-pressed="true" aria-label="Tronçon {i + 1}" style="flex: 1; height: 38px; border: 0; cursor: pointer; background: #B9DCC6; box-shadow: inset 0 0 0 2px {C["green"]}; '
        f'border-radius: {"9px 0 0 9px" if i == 0 else ("0 9px 9px 0" if i == 5 else "0")}"></button>'
        for i in range(6))
    zone = (f'<div style="display: flex; flex-direction: column; gap: 8px">'
            f'<div style="display: flex; justify-content: space-between; align-items: baseline"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">Zone de la planche · Au bout de l\'allée</span><span style="font-size: 13px; font-weight: 600">3 m sur 1,2 m</span></div>'
            f'<div role="group" aria-label="Raccourcis de zone" style="display: flex; padding: 4px; border-radius: 99px; background: {C["bg"]}">{parts}</div>'
            f'<div role="group" aria-label="Tronçons de 50 cm" style="display: flex; gap: 3px">{tsegs}</div></div>')

    left = card(
        f'<div style="display: flex; flex-direction: column; gap: 18px">'
        f'{lab(1, "Quel légume ?")}<div style="display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: 6px">{grid}</div>'
        f'{lab(2, "Comment tu démarres ?")}<div role="radiogroup" aria-label="Mode de démarrage" style="display: flex; flex-direction: column; gap: 8px">{mode_cards}</div>'
        f'{lab(3, "Quand plantes-tu ?")}'
        f'<label for="d-plant" style="display: flex; align-items: center; gap: 10px; padding: 0 14px; height: 50px; border-radius: 14px; background: {C["bg"]}">{icon("cal", 20, C["muted"])}'
        f'<input id="d-plant" value="Lundi 2 août 2027" style="border: 0; background: transparent; font: 600 16px Figtree, sans-serif; color: {C["ink"]}; width: 100%"></label>'
        f'</div>')
    right = card(f'<div style="display: flex; flex-direction: column; gap: 18px">{timeline}{zone}'
                 f'<div style="display: flex; justify-content: space-between; align-items: center; padding-top: 10px; border-top: 1px solid {C["line"]}">'
                 f'<span style="font-size: 14px; color: {C["muted"]}">≈ 40 pieds, 4 rangs × 10</span><strong style="font-size: 14px">1 barquette de plants</strong></div>'
                 f'{primary_btn("Planter 40 laitues", "../../POT-D01_ecrans-cles_v1.1/project/Main.dc.html", "check", block=True)}'
                 f'</div>', bg=C["card"])

    header = top_header("Nouvelle série", "Au bout de l'allée", close_href="../../POT-D01_ecrans-cles_v1.1/project/Main.dc.html")
    body = (f'<div style="{ROOTSTYLE}; min-height: 720px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">'
            f'<div style="flex: 1 1 520px; max-width: 620px">{left}</div><div style="flex: 1 1 380px; max-width: 420px">{right}</div></div></div>')
    return page("Nouvelle série — ordinateur", body, {"$preview": {"width": 1320, "height": 720}})


# ================= Artboard : E09 · Catalogue (ordinateur) =================
CULTURE_FICHES = {
    "tomate": dict(nom="Tomate", famille="Solanacées", abri="56 j", apres="75 j", duree="70 j", fen="1er mai → 15 juin", rang="80 cm", pied="60 cm", saison=False),
    "laitue": dict(nom="Laitue", famille="Astéracées", abri="28 j", apres="50 j", duree="20 j", fen="1er mars → 30 sept.", rang="30 cm", pied="30 cm", saison=True),
    "carotte": dict(nom="Carotte", famille="Apiacées", abri="Non semée à l'abri", apres="100 j", duree="40 j", fen="1er mars → 15 juil.", rang="25 cm", pied="4 cm", saison=True),
    "haricot": dict(nom="Haricot vert", famille="Fabacées", abri="Non semé à l'abri", apres="60 j", duree="30 j", fen="10 mai → 31 juil.", rang="50 cm", pied="10 cm", saison=False),
    "courgette": dict(nom="Courgette", famille="Cucurbitacées", abri="21 j", apres="50 j", duree="80 j", fen="1er mai → 30 juin", rang="100 cm", pied="100 cm", saison=False),
    "poireau": dict(nom="Poireau", famille="Amaryllidacées", abri="70 j", apres="120 j", duree="90 j", fen="15 mai → 31 juil.", rang="30 cm", pied="12 cm", saison=False),
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
                  + (f'<span style="font-size: 10.5px; font-weight: 700; padding: 2px 9px; border-radius: 99px; background: {C["good"]}; color: #FFFFFF">De saison</span>' if f_["saison"] else "")
                  + '</button>')
    families = ["Toutes", "Astéracées", "Apiacées", "Amaryllidacées", "Fabacées", "Cucurbitacées", "Solanacées"]
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
        f'De saison maintenant · 20 avril 2027</label></div>'
        f'<div style="display: flex; flex-wrap: wrap; gap: 8px">{fam_chips}</div>'
        f'<div style="display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px">{cards}</div>'
        f'<p style="margin: 0; font-size: 12.5px; color: {C["muted"]}">6 cultures en V1 (S02) · liste allongée après la V1.</p>'
        f'</div>')

    f_ = CULTURE_FICHES["laitue"]
    stages = "".join(f'<div style="display: flex; flex-direction: column; align-items: center; gap: 6px"><span style="display: grid; place-items: center; height: 84px; width: 84px; border-radius: 18px; background: {C["card"]}">{crop_icon("laitue", st, 68)}</span>'
                      f'<span style="font-size: 11.5px; color: {C["muted"]}">{n}</span></div>' for st, n in enumerate(["Semis", "Jeune pousse", "Adulte", "Récolte"]))
    rows = [("Famille", f_["famille"]), ("Durée sous abri", f_["abri"]), ("Jusqu'à la récolte", f_["apres"]),
            ("Durée de récolte", f_["duree"]), ("Fenêtre de mise en place", f_["fen"]), ("Espacement entre rangs", f_["rang"]), ("Espacement sur le rang", f_["pied"])]
    table = "".join(f'<div style="display: flex; justify-content: space-between; gap: 12px; padding: 10px 0; border-top: 1px solid {C["line"]}"><span style="font-size: 13.5px; color: {C["muted"]}">{k}</span><strong style="font-size: 13.5px">{v}</strong></div>' for k, v in rows)
    fiche_card = card(
        f'<div style="display: flex; flex-direction: column; gap: 16px">'
        f'<div style="display: flex; justify-content: space-between; align-items: flex-start"><div style="display: flex; flex-direction: column; gap: 2px">'
        f'<span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Fiche</span><h2 style="{H}; font-size: 24px">{f_["nom"]}</h2></div>'
        f'<span style="font-size: 10.5px; font-weight: 700; padding: 4px 10px; border-radius: 99px; background: {C["good"]}; color: #FFFFFF">De saison</span></div>'
        f'<div style="display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px">{stages}</div>'
        f'<div>{table}</div>'
        f'<p style="margin: 0; display: flex; gap: 8px; font-size: 12.5px; color: {C["muted"]}; align-items: flex-start">{icon("lock", 14, C["muted"])}<span>Fiche en lecture seule, schéma figé pour la V1 (S02).</span></p>'
        f'</div>', bg=C["card"])

    header = top_header(None, nav_active="Catalogue")
    body = (f'<div style="{ROOTSTYLE}; min-height: 620px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">'
            f'<div style="flex: 2 1 640px; min-width: 0">{list_card}</div><div style="flex: 1 1 360px; max-width: 420px">{fiche_card}</div></div></div>')
    return page("Catalogue — ordinateur", body, {"$preview": {"width": 1400, "height": 620}})


# ================= Réglages : contenu partagé (ordinateur + téléphone) =================
def row(ic, title, sub, right_html="", danger=False):
    col = C["crit"] if danger else C["ink"]
    return (f'<div style="display: flex; align-items: center; gap: 14px; padding: 14px 4px; border-top: 1px solid {C["line"]}">'
            f'<span style="flex: none; width: 38px; height: 38px; border-radius: 11px; background: {C["bg"] if not danger else "#FBE7E5"}; display: grid; place-items: center">{icon(ic, 18, col)}</span>'
            f'<div style="flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px"><strong style="font-size: 14.5px; color: {col}">{title}</strong><span style="font-size: 12.5px; color: {C["muted"]}">{sub}</span></div>'
            f'{right_html}</div>')


def reglages_sections(compact=False):
    allee = pill_group("Largeur d'allée minimale", ["30 cm", "50 cm", "80 cm"], 1, "Largeur d'allée")
    export_import = (
        row("doc", "Exporter le potager", "Un fichier à garder, ou pour passer à un autre appareil.",
            secondary_btn("Exporter", ic="export"))
        + row("upload", "Importer un potager", "Remplace le potager actuel après confirmation.",
              secondary_btn("Importer", ic="upload")))
    import_dialog = (
        f'<div style="display: flex; flex-direction: column; gap: 12px; padding: 16px; border-radius: 16px; background: #FDF1E4; border: 1px solid #F3D7AE">'
        f'<div style="display: flex; gap: 10px; align-items: flex-start">{icon("alert", 20, C["warn"])}<strong style="{H}; font-size: 15px">Le fichier importé est plus ancien</strong></div>'
        f'<p style="margin: 0; font-size: 13.5px; line-height: 1.5">« potager-export-2026-11-02.json » date du <strong>2 novembre 2026</strong>. Le potager de cet appareil a été modifié plus récemment, le <strong>14 novembre 2026</strong>.</p>'
        f'<p style="margin: 0; font-size: 13.5px">Il remplacera <strong>toutes</strong> les données actuelles.</p>'
        f'<div style="display: flex; gap: 10px; justify-content: flex-end">{secondary_btn("Annuler")}{primary_btn("Remplacer quand même", bg=C["warn"], fg="#3A2808")}</div></div>')
    plan = row("undo", "Vider le plan", "Retire toutes les planches de la saison en cours et des suivantes.", secondary_btn("Vider"))
    effacer = row("trash", "Effacer les données de l'appli", "Supprime tout le potager de cet appareil, après double confirmation.", secondary_btn("Effacer", ic="trash", fg=C["crit"]), danger=True)
    return allee, export_import, import_dialog, plan, effacer


# ================= Artboard : E10 · Réglages (ordinateur, panneau) =================
def e10_board():
    allee, export_import, import_dialog, plan, effacer = reglages_sections()
    backdrop = (f'<div style="flex: 1 1 820px; min-width: 0; position: relative">'
                f'<div style="background: {C["card"]}; border-radius: 32px; padding: 22px; filter: saturate(0.85); opacity: 0.5">'
                f'<div style="display: flex; align-items: center; gap: 10px; margin-bottom: 14px">{LOGO}<span style="{H}; font-size: 22px">Potagerie</span></div>'
                f'<div style="display: flex; justify-content: center">{flat_plan(cell=30)}</div></div>'
                f'<div aria-hidden="true" style="position: absolute; inset: 0; background: rgba(31,58,46,0.1); border-radius: 32px"></div></div>')
    panel = (
        f'<aside style="flex: 1 1 420px; max-width: 460px; min-width: 0; background: #FFFFFF; border-radius: 28px; padding: 26px; display: flex; flex-direction: column; gap: 18px; box-shadow: 0 20px 48px rgba(31,58,46,0.18)">'
        f'<div style="display: flex; justify-content: space-between; align-items: center"><h1 style="{H}; font-size: 22px">Réglages</h1>'
        f'<a href="../../POT-D01_ecrans-cles_v1.1/project/Main.dc.html" aria-label="Fermer" style="width: 36px; height: 36px; border-radius: 50%; background: {C["bg"]}; display: grid; place-items: center; text-decoration: none">{icon("close", 16)}</a></div>'
        f'{allee}'
        f'<div style="display: flex; flex-direction: column"><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}; padding-top: 6px">Données</span>{export_import}</div>'
        f'{import_dialog}'
        f'<div style="display: flex; flex-direction: column">{plan}{effacer}</div>'
        f'</aside>')
    body = (f'<div style="{ROOTSTYLE}; min-height: 820px">'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 28px 32px; align-items: stretch">{backdrop}{panel}</div></div>')
    return page("Réglages — ordinateur", body, {"$preview": {"width": 1440, "height": 820}})


# ================= Artboard : E10 · Réglages (téléphone, plein écran) =================
def e10_mobile_board():
    allee, export_import, import_dialog, plan, effacer = reglages_sections(compact=True)
    header = (f'<div style="display: flex; align-items: center; gap: 12px; padding: 16px 18px 10px">'
              f'<a href="../../POT-D01_ecrans-cles_v1.1/project/E05_potager_mobile.dc.html" aria-label="Retour" style="width: 38px; height: 38px; border-radius: 50%; border: 1px solid {C["line"]}; background: #FFFFFF; display: grid; place-items: center; text-decoration: none">{icon("back", 16)}</a>'
              f'<h1 style="{H}; font-size: 19px">Réglages</h1></div>')
    body = (
        f'<div style="{ROOTSTYLE}; width: 390px; height: 844px; box-sizing: border-box; display: flex; flex-direction: column; overflow-y: auto">'
        f'<span aria-hidden="true" style="align-self: center; width: 120px; height: 28px; margin-top: 4px; flex: none"></span>'
        f'{header}'
        f'<div style="flex: 1; padding: 0 18px 26px; display: flex; flex-direction: column; gap: 16px">'
        f'{allee}'
        f'<div><span style="font-size: 12px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: {C["muted"]}">Données</span>{export_import}</div>'
        f'{import_dialog}'
        f'<div>{plan}{effacer}</div>'
        f'</div>'
        f'<div aria-hidden="true" style="align-self: center; width: 134px; height: 5px; border-radius: 99px; background: {C["ink"]}; opacity: 0.25; margin: 6px 0 10px"></div>'
        f'</div>')
    return page("Réglages — téléphone", body, {"$preview": {"width": 390, "height": 844}})


# ================= Artboard : E11 · Export de la frise (ordinateur) =================
YEAR = 2027
MONTHS = ["Janv.", "Févr.", "Mars", "Avr.", "Mai", "Juin", "Juil.", "Août", "Sept.", "Oct.", "Nov.", "Déc."]
MDAYS = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]


def doy(md):
    m, d = int(md[:2]), int(md[3:])
    return sum(MDAYS[:m - 1]) + d - 1


def pct(md):
    return doy(md) / 365 * 100


EXPORT_SERIES = [
    # (planche, nom, culture, étiquette, mise en place, début récolte, fin [clippée au 31 déc si à cheval sur l'année suivante])
    ("A", "Fond du jardin", "carotte", "Carotte", "03-15", "06-23", "08-02"),
    ("B", "Les tomates", "tomate", "Tomate", "05-05", "07-19", "09-27"),
    ("B", "Les tomates", "laitue", "Laitue", "05-13", "07-02", "07-22"),
    ("C", "Les courgettes", "courgette", "Courgette", "05-01", "06-20", "09-08"),
    ("D", "Racines", "carotte", "Carotte", "03-15", "06-23", "08-02"),
    ("D", "Racines", "poireau", "Poireau", "07-10", "11-07", "12-31"),
    ("E", "Côté cabane", "haricot", "Haricot vert", "06-10", "08-09", "09-08"),
    ("F", "Carré poireaux", "poireau", "Poireau", "05-19", "09-16", "12-15"),
    ("G", "Carré salades", "laitue", "Laitue", "04-20", "06-09", "06-29"),
    ("H", "Au bout de l'allée", "laitue", "Laitue", "08-02", "09-21", "10-06"),
]


def export_gantt_preview():
    cols = " ".join(f"{d}fr" for d in MDAYS)
    months = "".join(f'<div style="font-size: 10px; font-weight: 600; letter-spacing: 0.03em; text-transform: uppercase; color: {C["muted"]}; padding: 0 0 6px 4px; border-left: 1px solid {C["line"]}">{m[:4]}</div>' for m in MONTHS)
    rows_by_planche = {}
    for pid, nom, crop, label, deb, rec, fin in EXPORT_SERIES:
        rows_by_planche.setdefault((pid, nom), []).append((crop, label, deb, rec, fin))
    labels, tracks = "", ""
    for (pid, nom), items in rows_by_planche.items():
        # une sous-ligne par série en cas de chevauchement (F-VIS-2), bac compris
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
        for ic, t, sub, sel in (("doc", "PDF", "A4 ou A3 paysage, à imprimer", True), ("image", "Image PNG", "À partager ou garder une trace", False)))

    periode = (f'<div style="display: flex; gap: 10px">{field("exp-d", "Du", "1 janv. 2027")}{field("exp-f", "Au", "31 déc. 2027")}</div>'
               f'<p style="margin: -4px 0 0; font-size: 12.5px; color: {C["muted"]}">Par défaut, l\'année civile du curseur.</p>')

    planches_chips = "".join(
        f'<label style="display: inline-flex; align-items: center; gap: 7px; padding: 7px 12px; border-radius: 99px; background: {C["bg"]}; font-size: 13px; font-weight: 600">'
        f'<span aria-hidden="true" style="width: 15px; height: 15px; border-radius: 4px; background: {C["green"]}"></span>{nom}</label>'
        for _, nom in sorted({(pid, nom) for pid, nom, *_ in EXPORT_SERIES}))
    planches_sel = (f'<div style="display: flex; justify-content: space-between; align-items: baseline"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">Planches</span>'
                    f'<a href="#" style="font-size: 12.5px; font-weight: 600">Tout sélectionner</a></div><div style="display: flex; flex-wrap: wrap; gap: 8px">{planches_chips}</div>')

    toggles = "".join(
        f'<label style="display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 10px 2px; border-top: 1px solid {C["line"]}">'
        f'<span style="font-size: 14px">{t}</span><span style="width: 36px; height: 21px; border-radius: 99px; background: {C["good"] if on else C["line"]}; position: relative; flex: none">'
        f'<span style="position: absolute; top: 2px; {"right" if on else "left"}: 2px; width: 17px; height: 17px; border-radius: 50%; background: #FFFFFF"></span></span></label>'
        for t, on in (("Phases (abri, croissance, récolte)", True), ("Repère « aujourd'hui »", True), ("Annotations", True), ("Calendrier des tâches", True)))

    form = card(
        f'<div style="display: flex; flex-direction: column; gap: 20px">'
        f'<h1 style="{H}; font-size: 24px">Exporter la frise</h1>'
        f'<div style="display: flex; gap: 10px">{fmt_opts}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 8px"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}">Période</span>{periode}</div>'
        f'<div style="display: flex; flex-direction: column; gap: 8px">{planches_sel}</div>'
        f'<div style="display: flex; flex-direction: column"><span style="font-size: 12px; font-weight: 700; color: {C["muted"]}; padding-top: 6px">Contenu</span>{toggles}</div>'
        f'{primary_btn("Exporter en PDF", ic="export", block=True)}'
        f'</div>')

    legend = "".join(f'<span style="display: inline-flex; align-items: center; gap: 7px; font-size: 12.5px; color: {C["muted"]}"><span style="width: 22px; height: 11px; border-radius: 4px; {bg}"></span>{t}</span>'
                      for bg, t in ((f"background: {mix('#E5533D', 0.45)}", "Croissance"), ("background: #E5533D", "Récolte")))
    preview = card(
        f'<div style="display: flex; flex-direction: column; gap: 14px">'
        f'<div style="display: flex; justify-content: space-between; align-items: baseline"><strong style="font-size: 14px">Aperçu</strong><span style="font-size: 12.5px; color: {C["muted"]}">Export du 14 nov. 2026</span></div>'
        f'<div style="padding: 16px; border-radius: 18px; background: #FFFFFF">'
        f'<div style="display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 10px"><strong style="{H}; font-size: 16px">Le potager de Tim</strong><span style="font-size: 12px; color: {C["muted"]}">Année 2027</span></div>'
        f'<div style="display: flex; gap: 16px; margin-bottom: 14px">'
        f'<div style="flex: none">{flat_plan(cell=11, labels=False)}</div>'
        f'<div style="flex: 1; min-width: 0; align-self: center">{export_gantt_preview()}</div></div>'
        f'<div style="display: flex; gap: 14px; flex-wrap: wrap; padding-top: 10px; border-top: 1px solid {C["line"]}">{legend}</div></div>'
        f'<p style="margin: 0; font-size: 12.5px; color: {C["muted"]}">Le PDF ajoute une miniature du plan et, si cochée, la liste mois par mois des tâches à faire.</p>'
        f'</div>', bg=C["card"])

    header = top_header(None, nav_active=None)
    body = (f'<div style="{ROOTSTYLE}; min-height: 600px">{header}'
            f'<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 0 32px 32px; align-items: flex-start">'
            f'<div style="flex: 1 1 400px; max-width: 440px">{form}</div><div style="flex: 2 1 640px; min-width: 0">{preview}</div></div></div>')
    return page("Export de la frise — ordinateur", body, {"$preview": {"width": 1440, "height": 600}})


# ================= Artboard : ET01 · États transverses (ordinateur) =================
def state_card(title, body_html, height=360):
    return (f'<div style="display: flex; flex-direction: column; gap: 12px; background: #FFFFFF; border-radius: 24px; padding: 22px; height: {height}px; box-sizing: border-box">'
            f'<strong style="{H}; font-size: 16px">{title}</strong><div style="flex: 1; min-width: 0; display: flex; flex-direction: column">{body_html}</div></div>')


def et01_board():
    # 1. Premier lancement, sans terrain
    s1 = (f'<div style="flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 14px; text-align: center; background: {C["card"]}; border-radius: 18px; padding: 20px">'
          f'{LOGO}<p style="margin: 0; font-size: 14px; color: {C["muted"]}; max-width: 320px">Pas encore de terrain : rien n\'est encore dessiné.</p>'
          f'{primary_btn("Créer mon terrain", ic="plus")}</div>')

    # 2. Frise vide
    cols = " ".join("1fr" for _ in MONTHS)
    months2 = "".join(f'<div style="font-size: 10px; font-weight: 600; text-transform: uppercase; color: {C["line"]}; border-left: 1px solid {C["line"]}; padding-left: 4px">{m[:4]}</div>' for m in MONTHS)
    s2 = (f'<div style="flex: 1; display: flex; flex-direction: column; gap: 10px">'
          f'<div style="display: grid; grid-template-columns: {cols}">{months2}</div>'
          f'<div style="flex: 1; border: 1px dashed {C["line"]}; border-radius: 14px; display: flex; align-items: center; justify-content: center; text-align: center; padding: 16px">'
          f'<p style="margin: 0; font-size: 13.5px; color: {C["muted"]}; max-width: 280px">Aucune série pour l\'instant.<br>Ajoute une planche dans l\'atelier, puis plante-la.</p></div></div>')

    # 3. Déplacement refusé (conflit, S03 T12)
    s3 = (f'<div style="flex: 1; display: flex; flex-direction: column; gap: 10px">'
          f'<div style="display: flex; justify-content: center">{flat_plan(cell=22, planches=[p for p in PLANCHES if p["id"] in ("B", "E")], gw=18, gh=6, highlight="B", drag_to=("E", 1, 3), labels=True)}</div>'
          f'<div style="display: flex; gap: 10px; align-items: flex-start; padding: 12px 14px; border-radius: 14px; background: #FBE7E5">{icon("alert", 18, C["crit"])}'
          f'<p style="margin: 0; font-size: 13px; line-height: 1.4; color: {C["accentInk"]}"><strong>Position refusée.</strong> « Côté cabane » chevaucherait « Les tomates ». La planche revient à sa place.</p></div></div>')

    # 4. Erreur d'écriture de fichier
    s4 = (f'<div style="flex: 1; display: flex; flex-direction: column; gap: 14px; align-items: center; justify-content: center; text-align: center; background: {C["card"]}; border-radius: 18px; padding: 20px">'
          f'<span style="display: inline-flex; align-items: center; gap: 8px; padding: 8px 16px; border-radius: 99px; background: #FBE7E5; color: {C["crit"]}; font-size: 13.5px; font-weight: 700">{icon("alert", 16, C["crit"])}Erreur d\'enregistrement</span>'
          f'<p style="margin: 0; font-size: 13.5px; color: {C["muted"]}; max-width: 300px">Le fichier n\'a pas pu être enregistré (disque plein ou dossier inaccessible). Rien n\'est perdu : la modification reste affichée.</p>'
          f'{secondary_btn("Réessayer", ic="undo")}</div>')

    grid = (f'<div style="display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 20px">'
            f'{state_card("① Premier lancement, sans terrain", s1)}{state_card("② Frise vide", s2)}'
            f'{state_card("③ Déplacement refusé (conflit, S03 T12)", s3)}{state_card("④ Erreur d\'écriture de fichier", s4)}</div>')

    body = (f'<div style="{ROOTSTYLE}; width: 1360px; box-sizing: border-box; padding: 40px">'
            f'<div style="display: flex; flex-direction: column; gap: 6px; margin-bottom: 24px"><span style="font-size: 13px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: {C["muted"]}">États transverses</span>'
            f'<h1 style="{H}; font-size: 30px">Quatre états qui peuvent survenir sur n\'importe quel écran</h1></div>{grid}</div>')
    return page("États transverses — ordinateur", body, {"$preview": {"width": 1360, "height": 920}})


BOARDS = {
    "Main.dc.html": e02_board,
    "E06_panneau-planche_desktop.dc.html": e06_board,
    "E07_serie_desktop.dc.html": e07_board,
    "E09_catalogue_desktop.dc.html": e09_board,
    "E10_reglages_desktop.dc.html": e10_board,
    "E10_reglages_mobile.dc.html": e10_mobile_board,
    "E11_export-frise_desktop.dc.html": e11_board,
    "ET01_etats_desktop.dc.html": et01_board,
}

if __name__ == "__main__":
    for name, fn in BOARDS.items():
        with open(os.path.join(ROOT, name), "w", encoding="utf-8") as f:
            f.write(fn())
    canvas = {
        "v": 3,
        "createdOnFiles": {"v": 1, "at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")},
        "title": "POT-D02 · Écrans complémentaires V1",
        "launch": {"view": "canvas"},
        "pages": [],
        "boards": {
            "Main.dc.html": {"x": 0, "y": 0, "w": 1320, "h": 900, "title": "① E02 · Création du terrain — ordinateur", "expand": "fill", "is_interactive": True},
            "E06_panneau-planche_desktop.dc.html": {"x": 1480, "y": 0, "w": 1440, "h": 760, "title": "④ E06 · Panneau planche — ordinateur", "expand": "fill", "is_interactive": True},
            "E07_serie_desktop.dc.html": {"x": 3080, "y": 0, "w": 1320, "h": 720, "title": "⑤ E07 · Nouvelle série — ordinateur", "expand": "fill", "is_interactive": True},
            "E09_catalogue_desktop.dc.html": {"x": 0, "y": 1000, "w": 1400, "h": 620, "title": "E09 · Catalogue — ordinateur", "expand": "fill", "is_interactive": True},
            "E10_reglages_desktop.dc.html": {"x": 1560, "y": 1000, "w": 1440, "h": 820, "title": "E10 · Réglages — ordinateur", "expand": "fill", "is_interactive": True},
            "E10_reglages_mobile.dc.html": {"x": 3160, "y": 1000, "w": 390, "h": 844, "title": "E10 · Réglages — téléphone", "is_interactive": True},
            "E11_export-frise_desktop.dc.html": {"x": 0, "y": 1780, "w": 1440, "h": 600, "title": "E11 · Export de la frise — ordinateur", "expand": "fill", "is_interactive": True},
            "ET01_etats_desktop.dc.html": {"x": 1600, "y": 1780, "w": 1360, "h": 920, "title": "ET01 · États transverses — ordinateur"},
        },
        "order": ["Main.dc.html", "E06_panneau-planche_desktop.dc.html", "E07_serie_desktop.dc.html", "E09_catalogue_desktop.dc.html",
                  "E10_reglages_desktop.dc.html", "E10_reglages_mobile.dc.html", "E11_export-frise_desktop.dc.html", "ET01_etats_desktop.dc.html"],
        "notes": {
            "titre": {"x": 0, "y": -300, "text": "Parcours 6.1 : de la création du terrain à la planification (suite dans D01)", "kind": "title1", "maxW": 4400},
            "lien1": {"x": 1340, "y": 40, "w": 160, "fill": "green", "size": "s",
                      "text": "① → D01 E03\n« Créer le terrain » ouvre l’atelier du plan (D01)."},
            "lien2": {"x": 2940, "y": 40, "w": 160, "fill": "green", "size": "s",
                      "text": "D01 E05 → ④\n« Modifier » dans le panneau simplifié de D01 ouvre le panneau complet.\n\n④ → ⑤\n« Planter ici »."},
            "lien3": {"x": 4540, "y": 40, "w": 160, "fill": "green", "size": "s",
                      "text": "⑤ → D01 E05\n« Planter … » revient au potager.\n\n④ → D01 E03\n« Déplacer »."},
            "titre2": {"x": 0, "y": 900, "text": "Écrans annexes (catalogue, réglages, export) — réglages aussi en téléphone", "kind": "title1", "maxW": 4000},
            "titre3": {"x": 0, "y": 1680, "text": "États transverses", "kind": "title1", "maxW": 1360},
            "hyp": {"x": 1600, "y": 2760, "w": 360, "fill": "orange",
                    "text": "Potager d’exemple : 8 planches sur 10, les 6 légumes de la V1 (S02), saison 2027.\nDates et quantités vérifiées par un script S03 (voir le README), pas calculées à la main.\nLe bouton Play de chaque feuille permet de suivre les liens, y compris vers D01."},
        },
        "designSystems": [],
    }
    with open(os.path.join(ROOT, "canvas.json"), "w", encoding="utf-8") as f:
        json.dump(canvas, f, ensure_ascii=False, indent=1)
    print("ok", sorted(os.listdir(ROOT)))
