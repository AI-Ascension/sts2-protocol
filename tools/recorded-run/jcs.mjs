// SPDX-License-Identifier: MIT
// Original bounded JCS writer. JSON tokens are emitted directly; no object reordering trick.
export function insist(ok, code) { if (!ok) throw new Error(code); }
function unicode(s) {
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c >= 0xd800 && c <= 0xdbff) {
      const next = s.charCodeAt(++i);
      insist(next >= 0xdc00 && next <= 0xdfff, "invalid_unicode");
    } else insist(c < 0xdc00 || c > 0xdfff, "invalid_unicode");
  }
  return JSON.stringify(s);
}
export function canonical(value) {
  let nodes = 0;
  function emit(v, depth) {
    insist(++nodes <= 100000 && depth <= 32, "json_limit");
    if (v === null) return "null";
    if (typeof v === "string") return unicode(v);
    if (typeof v === "boolean") return String(v);
    if (typeof v === "number") {
      insist(Number.isFinite(v), "invalid_number");
      return JSON.stringify(v);
    }
    insist(typeof v === "object", "invalid_json");
    if (Array.isArray(v)) return "[" + v.map(x => emit(x, depth + 1)).join(",") + "]";
    return "{" + Object.keys(v).sort().map(k => unicode(k) + ":" + emit(v[k], depth + 1)).join(",") + "}";
  }
  return emit(value, 0);
}
export function parseCanonical(bytes, maximum = 65536) {
  insist(bytes.length <= maximum, "json_bytes");
  let source;
  try { source = new TextDecoder("utf-8", {fatal: true, ignoreBOM: true}).decode(bytes); }
  catch { throw new Error("invalid_utf8"); }
  // Scan nesting before JSON.parse (bounded bytes alone do not bound nesting).
  let depth = 0, quoted = false, escaped = false;
  for (const c of source) {
    if (quoted) {
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === '"') quoted = false;
    } else if (c === '"') quoted = true;
    else if (c === "{" || c === "[") insist(++depth <= 32, "json_depth");
    else if (c === "}" || c === "]") depth--;
  }
  let value;
  try { value = JSON.parse(source); } catch { throw new Error("invalid_json"); }
  // Duplicate/escaped-equivalent keys cannot survive byte-for-byte canonical comparison.
  insist(canonical(value) === source, "noncanonical_json");
  return value;
}
