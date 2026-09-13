// SPDX-License-Identifier: MIT

import test from "node:test";
import assert from "node:assert/strict";
import {validate} from "./validate.mjs";

test("independent wire witness validates game-information vectors", () => {
  assert.deepEqual(validate(), {
    profile: "game-information-query-v1",
    schemaDigest: "376845b0c86b4afcd2c79ffba753eb7e7e416f5410da26b4dae970cfee2221d9",
    goldens: 8,
    invalidVectors: 15,
  });
});
