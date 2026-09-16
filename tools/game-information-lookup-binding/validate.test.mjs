// SPDX-License-Identifier: MIT

import test from "node:test";
import assert from "node:assert/strict";
import {validate} from "./validate.mjs";

test("independent wire witness validates lookup-binding vectors", () => {
  assert.deepEqual(validate(), {
    profile: "game-information-lookup-binding-v1",
    schemaDigest: "f10f9af01d6be1de104069ba842e7971971e88f27553e782e81174ee7aa1cd58",
    goldens: 5,
    invalidVectors: 12,
  });
});
