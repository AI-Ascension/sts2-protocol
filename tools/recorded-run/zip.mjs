// SPDX-License-Identifier: MIT
import {inflateRawSync, deflateRawSync} from "node:zlib";
import {insist} from "./jcs.mjs";
export const LIMITS = Object.freeze({archive: 16777216, extracted: 33554432,
  entry: 16777216, manifest: 262144, omissions: 1048576, records: 25000, record: 65536, files: 32});
export function crc32(bytes) {
  let crc = 0xffffffff;
  for (const b of bytes) {
    crc ^= b;
    for (let i = 0; i < 8; i++) crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0);
  }
  return (crc ^ 0xffffffff) >>> 0;
}
const pathOK = s => s.length <= 128 && !/[\r\n]/.test(s) && /^(?:[a-z0-9][a-z0-9_-]{0,31}\/)*[a-z0-9][a-z0-9_.-]{0,63}$/.test(s);
export function readZip(input) {
  // Bound byte views before conversion; respect the view, not the backing size.
  insist(input instanceof Uint8Array, "archive_input_type");
  insist(input.byteLength >= 22 && input.byteLength <= LIMITS.archive, "archive_limit");
  insist(!(input.buffer instanceof SharedArrayBuffer), "archive_shared_buffer");
  const b = Buffer.from(input.buffer, input.byteOffset, input.byteLength);
  const end = b.length - 22; // No comments, trailing bytes, ZIP64 or multi-disk.
  insist(b.readUInt32LE(end) === 0x06054b50, "zip_end");
  insist(b.readUInt16LE(end + 4) === 0 && b.readUInt16LE(end + 6) === 0 &&
    b.readUInt16LE(end + 20) === 0, "zip_disk_comment");
  const count = b.readUInt16LE(end + 10), start = b.readUInt32LE(end + 16);
  insist(count > 0 && count <= LIMITS.files && count === b.readUInt16LE(end + 8), "zip_count");
  insist(start + b.readUInt32LE(end + 12) === end, "zip_directory");
  let pos = start, localEnd = 0, total = 0;
  const entries = new Map();
  for (let i = 0; i < count; i++) {
    insist(pos + 46 <= end && b.readUInt32LE(pos) === 0x02014b50, "zip_central");
    const nameLength = b.readUInt16LE(pos + 28), extra = b.readUInt16LE(pos + 30);
    const comment = b.readUInt16LE(pos + 32), off = b.readUInt32LE(pos + 42);
    const flags = b.readUInt16LE(pos + 8), method = b.readUInt16LE(pos + 10);
    const packed = b.readUInt32LE(pos + 20), size = b.readUInt32LE(pos + 24);
    const crc = b.readUInt32LE(pos + 16), attrs = b.readUInt32LE(pos + 38);
    insist(pos + 46 + nameLength <= end && extra === 0 && comment === 0 &&
      b.readUInt16LE(pos + 34) === 0 && b.readUInt16LE(pos + 36) === 0, "zip_extra");
    insist(b.readUInt16LE(pos + 6) === 20 && (flags === 0 || flags === 0x800) &&
      (method === 0 || method === 8), "zip_features");
    const made = b.readUInt16LE(pos + 4);
    insist((made === 20 && attrs === 0) || (made === 0x314 && attrs === 0x81a40000), "zip_type");
    const nameBytes = b.subarray(pos + 46, pos + 46 + nameLength);
    insist(nameBytes.every(x => x < 128), "zip_name");
    const name = nameBytes.toString("ascii");
    insist(pathOK(name) && !entries.has(name), "zip_path");
    insist(off === localEnd && off + 30 <= start && b.readUInt32LE(off) === 0x04034b50, "zip_overlap");
    insist(b.readUInt16LE(off + 4) === 20 && b.readUInt16LE(off + 6) === flags &&
      b.readUInt16LE(off + 8) === method && b.readUInt32LE(off + 14) === crc &&
      b.readUInt32LE(off + 18) === packed && b.readUInt32LE(off + 22) === size &&
      b.readUInt16LE(off + 26) === nameLength && b.readUInt16LE(off + 28) === 0 &&
      b.readUInt32LE(off + 10) === b.readUInt32LE(pos + 12), "zip_local_mismatch");
    insist(b.subarray(off + 30, off + 30 + nameLength).equals(nameBytes), "zip_local_name");
    const data = off + 30 + nameLength;
    insist(data + packed <= start, "zip_bounds");
    const cap = Math.min(LIMITS.entry, LIMITS.extracted - total,
      name === "manifest.json" ? LIMITS.manifest : name === "reports/omissions.json" ? LIMITS.omissions : LIMITS.entry);
    insist(size <= cap, "zip_output_limit");
    let decoded;
    if (method === 0) {
      insist(packed === size, "zip_stored_size");
      decoded = b.subarray(data, data + packed);
    } else {
      try {
        const result = inflateRawSync(b.subarray(data, data + packed), {maxOutputLength: Math.max(1, Math.min(cap, size)), info: true});
        insist(result.engine.bytesWritten === packed, "zip_deflate_trailing");
        decoded = result.buffer;
      } catch { throw new Error("zip_inflate_limit_or_data"); }
    }
    insist(decoded.length === size && crc32(decoded) === crc, "zip_crc_size");
    total += decoded.length;
    entries.set(name, decoded);
    localEnd = data + packed;
    pos += 46 + nameLength;
  }
  insist(pos === end && localEnd === start, "zip_layout");
  return entries;
}
// Deterministic synthetic fixture packer, not a source recording exporter.
export function writeZip(entries, compress = false) {
  const local = [], central = []; let offset = 0;
  for (const [name, value] of [...entries].sort(([a], [b]) => a < b ? -1 : 1)) {
    const raw = Buffer.from(value), data = compress ? deflateRawSync(raw) : raw;
    const n = Buffer.from(name), crc = crc32(raw), method = compress ? 8 : 0;
    const h = Buffer.alloc(30), c = Buffer.alloc(46);
    h.writeUInt32LE(0x04034b50); h.writeUInt16LE(20, 4); h.writeUInt16LE(method, 8);
    h.writeUInt32LE(crc, 14); h.writeUInt32LE(data.length, 18); h.writeUInt32LE(raw.length, 22); h.writeUInt16LE(n.length, 26);
    c.writeUInt32LE(0x02014b50); c.writeUInt16LE(20, 4); c.writeUInt16LE(20, 6); c.writeUInt16LE(method, 10);
    c.writeUInt32LE(crc, 16); c.writeUInt32LE(data.length, 20); c.writeUInt32LE(raw.length, 24); c.writeUInt16LE(n.length, 28); c.writeUInt32LE(offset, 42);
    local.push(h, n, data); central.push(c, n); offset += h.length + n.length + data.length;
  }
  const directory = Buffer.concat(central), end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50); end.writeUInt16LE(entries.size, 8); end.writeUInt16LE(entries.size, 10);
  end.writeUInt32LE(directory.length, 12); end.writeUInt32LE(offset, 16);
  return Buffer.concat([...local, directory, end]);
}
