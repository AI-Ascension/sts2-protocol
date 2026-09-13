// SPDX-License-Identifier: MIT

import test from "node:test";
import assert from "node:assert/strict";
import {validate} from "./validate.mjs";

test("independent wire witness validates game-information vectors", () => {
  assert.deepEqual(validate(), {
    profile: "game-information-query-v1",
    schemaDigest: "e5ba81b0520687cf59db6a94aea3b38606e86300f6eb2b0e858f55704e62f76c",
    goldens: 8,
    invalidVectors: 15,
  });
});
