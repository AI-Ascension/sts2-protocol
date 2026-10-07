# game-facts-reference-v1 candidate bundle

This directory contains a reviewable, inert draft for transporting typed rule
inputs. It does not read the game, calculate rules, authenticate a caller, or
claim that any rule is supported by a real build or mode.

`schema.json` is byte-identical to the standalone source schema. The eleven
goldens demonstrate closed message shapes, explicit unsupported outcomes,
static and live reference binding, and safe integer inputs. The synthetic live
vector keeps multiplier numerator and denominator as separate integer inputs;
it defines no formula or operation order.

The capability vector with a null snapshot policy represents static-only
availability. The live policy vector and all values/build labels in goldens are
synthetic examples, not producer claims. The manifest has no adopted consumers.

Verify bundle bytes against `SHA256SUMS`. The character bounds in JSON Schema
do not measure UTF-8 bytes; the typed codec applies the stricter byte bound.
Responses that echo a query must be joined to the original request with the
correlation and query comparison API before being treated as that request's
answer.
