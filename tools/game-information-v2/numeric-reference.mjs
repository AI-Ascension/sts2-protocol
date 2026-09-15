// SPDX-License-Identifier: MIT
// Independent exact oracle: inspect original JSON.parse reviver source, never its rounded Number.
import {readFileSync} from "node:fs";
import {fileURLToPath} from "node:url";
import {resolve} from "node:path";

const MAX_SAFE = 9007199254740991n;
const canonical = (value) => Array.isArray(value) ? `[${value.map(canonical).join(",")}]`
  : value !== null && typeof value === "object" ? `{${Object.keys(value).sort().map(
    (key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}` : JSON.stringify(value);

function exactInteger(source) {
  const parts = /^(-?)(0|[1-9][0-9]*)(?:\.([0-9]+))?(?:[eE]([+-]?[0-9]+))?$/.exec(source);
  if (!parts) throw new Error("malformed_number");
  const digits = (parts[2] + (parts[3] ?? "")).replace(/^0+/, "");
  if (digits.length === 0) return 0;
  const exponentText = parts[4] ?? "0";
  const exponentDigits = exponentText.replace(/^[+-]/, "").replace(/^0+/, "");
  // Exponents beyond the input's possible coefficient shift cannot yield an in-range nonzero integer.
  if (exponentDigits.length > String(source.length + 17).length) throw new Error("numeric_range");
  const shift = BigInt(exponentText) - BigInt((parts[3] ?? "").length);
  const width = BigInt(digits.length) + shift;
  if (width < 1n || width > 16n) throw new Error("numeric_range");
  let integerDigits = digits;
  if (shift < 0n) {
    const cut = Number(width);
    if (!/^0*$/.test(digits.slice(cut))) throw new Error("non_integral");
    integerDigits = digits.slice(0, cut);
  }
  let magnitude = BigInt(integerDigits);
  if (shift > 0n) magnitude *= 10n ** shift;
  if (magnitude > MAX_SAFE) throw new Error("unsafe_integer");
  return Number(parts[1] ? -magnitude : magnitude);
}

export function numericReference(raw) {
  try {
    const value = JSON.parse(raw, (_key, value, context) => {
      if (typeof value !== "number") return value;
      if (typeof context?.source !== "string") throw new Error("reviver_source_unavailable");
      return exactInteger(context.source);
    });
    return {valid: true, canonical: canonical(value)};
  } catch (error) {
    if (error.message === "reviver_source_unavailable") throw error;
    return {valid: false, canonical: null};
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const inputs = JSON.parse(readFileSync(0, "utf8"));
  process.stdout.write(JSON.stringify(inputs.map(numericReference)));
}
