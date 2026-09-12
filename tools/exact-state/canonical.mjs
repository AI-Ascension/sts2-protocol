// SPDX-License-Identifier: MIT
// Independent bounded witness for the asc-jcs-state-v1 restricted canonical profile.
// This is not a general RFC 8785 implementation: ASCII schema keys, safe integers, and
// explicit rejects remove the JCS key-ordering and binary64 edge cases. It is written
// against the contract, not transliterated from another implementation.

import {createHash} from "node:crypto";

export const LIMITS = {inputBytes: 16 * 1024 * 1024, canonicalBytes: 16 * 1024 * 1024, depth: 64};
export const MAX_SAFE_INTEGER = 9007199254740991;
export const STATE_PREFIX = "asc-state:v1:sha256:";
export const CHECKPOINT_PREFIX = "asc-checkpoint:v1:sha256:";
export const BLOB_PREFIX = "sha256:";
const STATE_DOMAIN = Buffer.from("AI-ASCENSION/EXACT-STATE/v1\0", "utf8");
const CHECKPOINT_DOMAIN = Buffer.from("AI-ASCENSION/CHECKPOINT/v1\0", "utf8");
const KEY = /^[a-z][a-z0-9_]*$/;
const INTEGER = /^-?(0|[1-9][0-9]*)$/;

function fail(code) { throw new Error(code); }

function assertScalar(text) {
  for (let index = 0; index < text.length; index += 1) {
    const unit = text.charCodeAt(index);
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = text.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) fail("lone_surrogate");
      index += 1;
    } else if (unit >= 0xdc00 && unit <= 0xdfff) fail("lone_surrogate");
  }
}

class Reader {
  constructor(text) { this.text = text; this.pos = 0; }
  peek() { return this.pos < this.text.length ? this.text[this.pos] : undefined; }
  take(expected) { if (this.text[this.pos] !== expected) fail("syntax"); this.pos += 1; }
  skip() { while (this.pos < this.text.length && " \t\n\r".includes(this.text[this.pos])) this.pos += 1; }
  value(depth) {
    this.skip();
    const char = this.peek();
    if (char === "{") return this.object(depth);
    if (char === "[") return this.array(depth);
    if (char === '"') return this.string();
    for (const [word, value] of [["true", true], ["false", false], ["null", null]]) {
      if (this.text.startsWith(word, this.pos)) { this.pos += word.length; return value; }
    }
    if (char === "-" || (char >= "0" && char <= "9")) return this.integer();
    return fail("syntax");
  }
  object(depth) {
    if (depth >= LIMITS.depth) fail("depth");
    this.take("{");
    const result = {};
    const seen = new Set();
    this.skip();
    if (this.peek() === "}") { this.pos += 1; return result; }
    for (;;) {
      this.skip();
      if (this.peek() !== '"') fail("syntax");
      const key = this.string();
      if (!KEY.test(key)) fail("key");
      if (seen.has(key)) fail("duplicate_key");
      seen.add(key);
      this.skip();
      this.take(":");
      result[key] = this.value(depth + 1);
      this.skip();
      if (this.peek() === ",") { this.pos += 1; continue; }
      this.take("}");
      return result;
    }
  }
  array(depth) {
    if (depth >= LIMITS.depth) fail("depth");
    this.take("[");
    const result = [];
    this.skip();
    if (this.peek() === "]") { this.pos += 1; return result; }
    for (;;) {
      result.push(this.value(depth + 1));
      this.skip();
      if (this.peek() === ",") { this.pos += 1; continue; }
      this.take("]");
      return result;
    }
  }
  string() {
    this.take('"');
    let out = "";
    for (;;) {
      const char = this.peek();
      if (char === undefined) fail("syntax");
      if (char === '"') { this.pos += 1; assertScalar(out); return out; }
      if (char === "\\") {
        this.pos += 1;
        const escape = this.peek();
        this.pos += 1;
        const simple = { '"': '"', "\\": "\\", "/": "/", b: "\b", f: "\f", n: "\n", r: "\r", t: "\t" };
        if (escape in simple) { out += simple[escape]; continue; }
        if (escape !== "u") fail("syntax");
        const unit = this.hex4();
        if (unit >= 0xd800 && unit <= 0xdbff) {
          if (this.text[this.pos] !== "\\" || this.text[this.pos + 1] !== "u") fail("syntax");
          this.pos += 2;
          const low = this.hex4();
          if (!(low >= 0xdc00 && low <= 0xdfff)) fail("syntax");
          out += String.fromCharCode(unit, low);
        } else if (unit >= 0xdc00 && unit <= 0xdfff) {
          fail("syntax");
        } else {
          out += String.fromCharCode(unit);
        }
        continue;
      }
      if (char.charCodeAt(0) < 0x20) fail("syntax");
      out += char;
      this.pos += 1;
    }
  }
  hex4() {
    const digits = this.text.slice(this.pos, this.pos + 4);
    if (!/^[0-9a-fA-F]{4}$/.test(digits)) fail("syntax");
    this.pos += 4;
    return Number.parseInt(digits, 16);
  }
  integer() {
    const start = this.pos;
    while (this.pos < this.text.length && /[0-9.eE+-]/.test(this.text[this.pos])) this.pos += 1;
    const token = this.text.slice(start, this.pos);
    if (!INTEGER.test(token) || token === "-0" || token.length > 17) fail("number_token");
    const value = Number(token);
    if (!Number.isSafeInteger(value)) fail("integer_range");
    return value;
  }
}

function escape(text) {
  let out = '"';
  for (const char of text) {
    const code = char.codePointAt(0);
    if (char === '"') out += '\\"';
    else if (char === "\\") out += "\\\\";
    else if (code === 8) out += "\\b";
    else if (code === 9) out += "\\t";
    else if (code === 10) out += "\\n";
    else if (code === 12) out += "\\f";
    else if (code === 13) out += "\\r";
    else if (code < 0x20) out += "\\u" + code.toString(16).padStart(4, "0");
    else out += char;
  }
  return out + '"';
}

function emit(value, depth, budget) {
  if (value === null) return "null";
  if (typeof value === "boolean") return String(value);
  if (typeof value === "number") {
    if (!Number.isSafeInteger(value) || Object.is(value, -0)) fail("integer_range");
    return String(value);
  }
  if (typeof value === "string") {
    assertScalar(value);
    if (value.length > LIMITS.canonicalBytes) fail("string_bytes");
    return escape(value);
  }
  if (Array.isArray(value)) {
    if (depth >= LIMITS.depth) fail("depth");
    return "[" + value.map((item) => emit(item, depth + 1, budget)).join(",") + "]";
  }
  if (typeof value === "object") {
    if (depth >= LIMITS.depth) fail("depth");
    const keys = Object.keys(value);
    for (const key of keys) if (!KEY.test(key)) fail("key");
    return "{" + keys.sort().map((key) => escape(key) + ":" + emit(value[key], depth + 1, budget)).join(",") + "}";
  }
  return fail("unsupported_type");
}

export function parseProfile(text) {
  if (typeof text !== "string") fail("input_type");
  if (Buffer.byteLength(text, "utf8") > LIMITS.inputBytes) fail("input_bytes");
  if (text.startsWith("\ufeff")) fail("bom");
  let depth = 0;
  let quoted = false;
  let escaped = false;
  for (const char of text) {
    if (quoted) {
      if (escaped) escaped = false;
      else if (char === "\\") escaped = true;
      else if (char === '"') quoted = false;
      continue;
    }
    if (char === '"') quoted = true;
    else if (char === "{" || char === "[") { depth += 1; if (depth > LIMITS.depth) fail("depth"); }
    else if (char === "}" || char === "]") { depth -= 1; if (depth < 0) fail("syntax"); }
  }
  const reader = new Reader(text);
  const value = reader.value(0);
  reader.skip();
  if (reader.pos !== text.length) fail("trailing_data");
  return value;
}

export function canonicalize(text) {
  const bytes = emit(parseProfile(text), 0, 0);
  if (Buffer.byteLength(bytes, "utf8") > LIMITS.canonicalBytes) fail("canonical_bytes");
  return bytes;
}

export function sha256(bytes) { return createHash("sha256").update(bytes).digest(); }
export function stateId(text) {
  return STATE_PREFIX + createHash("sha256").update(STATE_DOMAIN).update(Buffer.from(canonicalize(text), "utf8")).digest("hex");
}
export function checkpointId(text) {
  return CHECKPOINT_PREFIX + createHash("sha256").update(CHECKPOINT_DOMAIN).update(Buffer.from(canonicalize(text), "utf8")).digest("hex");
}
export function blobDigest(bytes) { return BLOB_PREFIX + sha256(bytes).toString("hex"); }
export function canonicalHex(text) { return Buffer.from(canonicalize(text), "utf8").toString("hex"); }
