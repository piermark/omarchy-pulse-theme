// Makes native/src/cities_data.rs from the published web page: the landmarks of every city (pulse.js's LM and CITIES)
// are run as they are, on a context that records their outlines, so pulse-native draws the very same shapes.
// Run from the repo's root: node native/tools/gen-cities.js > native/src/cities_data.rs
const fs = require("fs");
const src = fs.readFileSync("extras/pulse/web/pulse.js", "utf8").split("\n");
const from = src.findIndex(l => l.includes("function sym(c, x, y, w, h, prof)"));
const to = src.findIndex(l => l.trim().startsWith('var city = "", cityTops = [];'));
if (from < 0 || to < 0) throw new Error("pulse.js changed: the landmarks were not found");
const rndLine = src.find(l => /function rnd\(|rnd = function/.test(l));
const code = (rndLine || "") + "\n" + src.slice(from, to).join("\n") + "\nreturn { LM: LM, CITIES: CITIES, STROKED: STROKED };";
const env = new Function("rgba", "COL", "mix", "H", "W", code)(() => "", {}, (a) => a, 1, 1);
function recorder() {
  const subs = []; let cur = null;
  const p = (x, y) => { if (!cur) { cur = []; subs.push(cur); } cur.push([x, y]); };
  const flatArc = (cx, cy, rx, ry, rot, a0, a1, ccw) => {
    let d = a1 - a0;
    if (!ccw) { while (d < 0) d += Math.PI * 2; if (d > Math.PI * 2) d = Math.PI * 2; } else { while (d > 0) d -= Math.PI * 2; if (d < -Math.PI * 2) d = -Math.PI * 2; }
    const n = Math.max(4, Math.ceil(Math.abs(d) / (Math.PI * 2) * 48)), cs = Math.cos(rot), sn = Math.sin(rot);
    for (let i = 0; i <= n; i++) { const a = a0 + d * i / n, x = Math.cos(a) * rx, y = Math.sin(a) * ry; p(cx + x * cs - y * sn, cy + x * sn + y * cs); }
  };
  return {
    subs,
    moveTo(x, y) { cur = [[x, y]]; subs.push(cur); },
    lineTo(x, y) { p(x, y); },
    closePath() { if (cur && cur.length) { cur.push([cur[0][0], cur[0][1]]); const s = cur[0]; cur = [[s[0], s[1]]]; subs.push(cur); } },
    rect(x, y, w, h) { subs.push([[x, y], [x + w, y], [x + w, y + h], [x, y + h], [x, y]]); cur = null; },
    arc(cx, cy, r, a0, a1, ccw) { flatArc(cx, cy, r, r, 0, a0, a1, !!ccw); },
    ellipse(cx, cy, rx, ry, rot, a0, a1, ccw) { flatArc(cx, cy, rx, ry, rot, a0, a1, !!ccw); },
    bezierCurveTo(x1, y1, x2, y2, x, y) { const s = cur[cur.length - 1]; for (let i = 1; i <= 14; i++) { const t = i / 14, u = 1 - t; p(u * u * u * s[0] + 3 * u * u * t * x1 + 3 * u * t * t * x2 + t * t * t * x, u * u * u * s[1] + 3 * u * u * t * y1 + 3 * u * t * t * y2 + t * t * t * y); } },
    quadraticCurveTo(x1, y1, x, y) { const s = cur[cur.length - 1]; for (let i = 1; i <= 10; i++) { const t = i / 10, u = 1 - t; p(u * u * s[0] + 2 * u * t * x1 + t * t * x, u * u * s[1] + 2 * u * t * y1 + t * t * y); } },
  };
}
const f = n => { const s = (Math.round(n * 1e5) / 1e5).toString(); return /[.e]/.test(s) ? s : s + ".0"; };
let out = "//! Made by native/tools/gen-cities.js from extras/pulse/web/pulse.js (LM and CITIES): do not edit by hand.\n" +
  "//! Each landmark: (name, x from -1 to 1 beside the core, width and height in screen heights, how it is drawn, its outlines\n" +
  "//! in screen heights from its foot on the horizon).\n\n" +
  "pub struct Landmark {\n    pub name: &'static str,\n    pub x: f32,\n    pub w: f32,\n    pub h: f32,\n    /// drawn as a line (a wheel, a bridge), not filled\n    pub stroked: bool,\n    /// a hill, a mesa, a volcano: no windows\n    pub nat: bool,\n    pub flat: bool,\n    /// tall enough for a red light on its top\n    pub top: bool,\n    pub paths: &'static [&'static [(f32, f32)]],\n}\n\n";
let table = "pub fn city(name: &str) -> Option<&'static [Landmark]> {\n    Some(match name {\n";
let k = 0;
for (const [name, list] of Object.entries(env.CITIES)) {
  const id = "C" + (k++);
  out += `static ${id}: &[Landmark] = &[\n`;
  for (const l of list) {
    const nat = l[0] === "hill" || l[0] === "mesa" || l[0] === "volcano", flat = nat || l[0] === "blocks" || l[0] === "houses";
    const h = flat ? l[2] * 1.1 : 0.05 + l[2] * 1.2, w = l[3] * (flat ? 1 : 1.2);
    const c = recorder();
    env.LM[l[0]](c, 0, 0, w, h, l[4]);
    const subs = c.subs.filter(s => s.length > 1);
    out += `    Landmark { name: ${JSON.stringify(l[0])}, x: ${f(l[1])}_f32, w: ${f(w)}_f32, h: ${f(h)}_f32, stroked: ${!!env.STROKED[l[0]]}, nat: ${nat}, flat: ${flat}, top: ${!flat && l[2] > 0.15}, paths: &[` +
      subs.map(s => "&[" + s.map(q => `(${f(q[0])}, ${f(q[1])})`).join(", ") + "]").join(", ") + "] },\n";
  }
  out += "];\n";
  table += `        ${JSON.stringify(name)} => ${id},\n`;
}
out += "\n" + table + "        _ => return None,\n    })\n}\n";
process.stdout.write(out);
