// SPDX-License-Identifier: MIT
// Closed evaluator for exactly the JSON Schema keywords used by this artifact.
// The Rust conformance test independently compiles the schema with Draft 2020-12.
import {canonical, insist} from "./jcs.mjs";
export function validateSchema(root, value, definition) {
  // The recording profile has no free text: token/digest/numeric strings must
  // not carry whitespace/control characters, even where regex $ matches before LF.
  function inertTokens(v) {
    if (typeof v === "string") insist(!/[\u0000-\u0020\u007f-\u009f]/u.test(v), "profile_control_character");
    else if (v && typeof v === "object") Object.values(v).forEach(inertTokens);
  }
  inertTokens(value);
  function check(s, v) {
    if (s.$ref) return check(root.$defs[s.$ref.split("/").at(-1)], v);
    if (s.oneOf && s.oneOf.filter(x => check(x, v)).length !== 1) return false;
    if (s.anyOf && !s.anyOf.some(x => check(x, v))) return false;
    if ("const" in s && canonical(v) !== canonical(s.const)) return false;
    if (s.enum && !s.enum.some(x => canonical(x) === canonical(v))) return false;
    if (s.type === "null" && v !== null) return false;
    if (s.type === "boolean" && typeof v !== "boolean") return false;
    if (s.type === "integer" && (!Number.isSafeInteger(v) || v < s.minimum || v > s.maximum)) return false;
    if (s.type === "string") {
      if (typeof v !== "string") return false;
      const n = [...v].length;
      if (n < (s.minLength ?? 0) || n > (s.maxLength ?? Infinity)) return false;
      if (s.pattern && !new RegExp(s.pattern, "u").test(v)) return false;
    }
    if (s.type === "array") {
      if (!Array.isArray(v) || v.length < (s.minItems ?? 0) || v.length > (s.maxItems ?? Infinity)) return false;
      if (s.uniqueItems && new Set(v.map(canonical)).size !== v.length) return false;
      if (!v.every(x => check(s.items, x))) return false;
    }
    if (s.type === "object") {
      if (!v || typeof v !== "object" || Array.isArray(v)) return false;
      if (s.required.some(k => !Object.hasOwn(v, k))) return false;
      if (Object.keys(v).some(k => !Object.hasOwn(s.properties, k))) return false;
      if (!Object.entries(v).every(([k, x]) => check(s.properties[k], x))) return false;
    }
    return true;
  }
  insist(check(root.$defs[definition], value), "schema_" + definition);
}
