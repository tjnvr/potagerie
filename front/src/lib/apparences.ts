// Generated with the catalogue (S02 v0.3): for each crop without its own
// drawing, the drawn crop it looks like in the diorama and the icon, and its
// colour on the timeline.

export const APPARENCES: Record<string, { sosie: string; couleur: string }> = {
  "aubergine": { sosie: "tomate", couleur: "#6B3A7A" },
  "poivron": { sosie: "tomate", couleur: "#D93B2B" },
  "piment": { sosie: "tomate", couleur: "#B8241C" },
  "tomate-cerise": { sosie: "tomate", couleur: "#F06A4F" },
  "pomme-de-terre": { sosie: "haricot-vert", couleur: "#C9A66B" },
  "physalis": { sosie: "tomate", couleur: "#E8A33D" },
  "laitue-romaine": { sosie: "laitue", couleur: "#7DB548" },
  "chicoree-frisee": { sosie: "laitue", couleur: "#A6CF6A" },
  "scarole": { sosie: "laitue", couleur: "#B5D67C" },
  "endive": { sosie: "carotte", couleur: "#E3E0B0" },
  "artichaut": { sosie: "courgette", couleur: "#7A9A8C" },
  "salsifis": { sosie: "carotte", couleur: "#8C7A5B" },
  "topinambour": { sosie: "haricot-vert", couleur: "#D8B04A" },
  "panais": { sosie: "carotte", couleur: "#E8D9A8" },
  "celeri-rave": { sosie: "carotte", couleur: "#BFCF9A" },
  "celeri-branche": { sosie: "carotte", couleur: "#9CC27A" },
  "fenouil": { sosie: "carotte", couleur: "#CFE3A0" },
  "persil": { sosie: "carotte", couleur: "#3E8E3A" },
  "cerfeuil": { sosie: "carotte", couleur: "#78B060" },
  "coriandre": { sosie: "carotte", couleur: "#5EA052" },
  "aneth": { sosie: "carotte", couleur: "#A9C95A" },
  "haricot-a-ecosser": { sosie: "haricot-vert", couleur: "#B88A4A" },
  "haricot-a-rames": { sosie: "haricot-vert", couleur: "#4F9A3A" },
  "petit-pois": { sosie: "haricot-vert", couleur: "#7CC46A" },
  "pois-gourmand": { sosie: "haricot-vert", couleur: "#9AD27F" },
  "feve": { sosie: "haricot-vert", couleur: "#6E8F4E" },
  "concombre": { sosie: "courgette", couleur: "#2F7A3E" },
  "cornichon": { sosie: "courgette", couleur: "#4A8A35" },
  "melon": { sosie: "courgette", couleur: "#E9B872" },
  "pasteque": { sosie: "courgette", couleur: "#2E6B3A" },
  "potiron": { sosie: "courgette", couleur: "#E3782A" },
  "potimarron": { sosie: "courgette", couleur: "#D9602A" },
  "courge-butternut": { sosie: "courgette", couleur: "#E8B04F" },
  "patisson": { sosie: "courgette", couleur: "#F2DE7A" },
  "oignon": { sosie: "poireau", couleur: "#C98A4B" },
  "oignon-blanc": { sosie: "poireau", couleur: "#E9E4D0" },
  "echalote": { sosie: "poireau", couleur: "#B5654A" },
  "ail": { sosie: "poireau", couleur: "#E6DCCB" },
  "ciboulette": { sosie: "poireau", couleur: "#5C9E4F" },
  "chou-pomme": { sosie: "laitue", couleur: "#8FB8A0" },
  "chou-fleur": { sosie: "laitue", couleur: "#EDE6C8" },
  "brocoli": { sosie: "laitue", couleur: "#3F7D4E" },
  "chou-de-bruxelles": { sosie: "laitue", couleur: "#5E8F4A" },
  "chou-kale": { sosie: "laitue", couleur: "#2F5E45" },
  "chou-rave": { sosie: "laitue", couleur: "#A7C7A0" },
  "chou-chinois": { sosie: "laitue", couleur: "#C7DE9A" },
  "pak-choi": { sosie: "laitue", couleur: "#6FAE6A" },
  "radis": { sosie: "carotte", couleur: "#E0445E" },
  "radis-noir": { sosie: "carotte", couleur: "#3A3A3A" },
  "navet": { sosie: "carotte", couleur: "#B486C2" },
  "rutabaga": { sosie: "carotte", couleur: "#C9A46B" },
  "roquette": { sosie: "laitue", couleur: "#4E8A3A" },
  "betterave": { sosie: "carotte", couleur: "#8E2A4A" },
  "blette": { sosie: "laitue", couleur: "#C8D84A" },
  "epinard": { sosie: "laitue", couleur: "#2E6E3A" },
  "basilic": { sosie: "laitue", couleur: "#4FA35A" },
  "mache": { sosie: "laitue", couleur: "#3F7A3A" },
  "mais-doux": { sosie: "poireau", couleur: "#E8C547" },
  "patate-douce": { sosie: "courgette", couleur: "#C56A3A" },
  "gombo": { sosie: "tomate", couleur: "#7FA34A" },
  "pourpier": { sosie: "laitue", couleur: "#8DBA5A" },
};

/** The six crops drawn by D01 (plants2.py), whose ids name the symbols. */
const DESSINES = new Set(["tomate", "laitue", "carotte", "haricot-vert", "courgette", "poireau"]);

/** The drawn crop that stands for `culture` (`haricot_vert` or `haricot-vert`). */
export function dessinDe(culture: string): string {
  const id = culture.replaceAll("_", "-");
  if (DESSINES.has(id)) return id;
  return APPARENCES[id]?.sosie ?? "laitue";
}
