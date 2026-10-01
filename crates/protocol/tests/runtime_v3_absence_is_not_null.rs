// SPDX-License-Identifier: MIT

//! Conformance for the rule that an optional member is *absent*, never `null`.
//!
//! "Absence is never a plausible value" (ADR 0045) is only enforced if absence and an explicit JSON
//! `null` are told apart, and serde's derived `Option<T>` cannot do that on its own: `#[serde(default)]`
//! supplies absence, and the derived `Option` handling supplies `None` for `null` too. Both defects
//! here widened or narrowed the boundary in opposite directions, so each has its own case.

mod support {
    pub mod runtime_v3_parity {
        include!("support/runtime_v3_parity.rs");
    }
}

use serde_json::{Value, json};
use support::runtime_v3_parity::{assert_parity, full_player, with_legal_actions, with_player};

#[test]
fn use_potion_requires_its_target_member_even_when_the_schema_could_omit_it() {
    // `target_id` is required on the wire, exactly as on `play_card`, and a potion with no target
    // says so with null rather than by leaving the member out. The schema listed only `kind` and
    // `potion_id`, so it admitted a document the parser refused: the two disagreed about the same
    // message, in the direction that loses fail-closed behaviour.
    let document = with_legal_actions(json!([{
        "action_id": "potion.act",
        "action": { "kind": "use_potion", "potion_id": "potion:2:Block" },
    }]));
    assert_parity(&document, false, "use_potion with target_id omitted");
}

#[test]
fn an_explicit_null_on_any_optional_member_is_refused() {
    // Every optional member below is `Option<T>`, so an explicit null decodes to `None` just as an
    // absent member does, while the schema's optional rules refuse it. Without a strict
    // deserializer the parser admitted documents the schema rejected.
    let cases: [(&str, Value); 9] = [
        ("player.relics", {
            let mut player = full_player();
            player["relics"] = Value::Null;
            player
        }),
        ("player.potions", {
            let mut player = full_player();
            player["potions"] = Value::Null;
            player
        }),
        ("player.potion_slots", {
            let mut player = full_player();
            player["potion_slots"] = Value::Null;
            player
        }),
        ("player.max_potion_slots", {
            let mut player = full_player();
            player["max_potion_slots"] = Value::Null;
            player
        }),
        ("relic.description", {
            let mut player = full_player();
            player["relics"] = json!([{
                "relic_id": "relic:1:Anchor",
                "name": "Anchor",
                "description": Value::Null,
            }]);
            player
        }),
        ("potion.slot", {
            let mut player = full_player();
            player["potions"] = json!([{
                "potion_id": "potion:1:Fire",
                "name": "Fire",
                "slot": Value::Null,
            }]);
            player
        }),
        ("potion.usable", {
            let mut player = full_player();
            player["potions"] = json!([{
                "potion_id": "potion:1:Fire",
                "name": "Fire",
                "usable": Value::Null,
            }]);
            player
        }),
        ("potion.target_mode", {
            let mut player = full_player();
            player["potions"] = json!([{
                "potion_id": "potion:1:Fire",
                "name": "Fire",
                "target_mode": Value::Null,
            }]);
            player
        }),
        ("potion.description", {
            let mut player = full_player();
            player["potions"] = json!([{
                "potion_id": "potion:1:Fire",
                "name": "Fire",
                "description": Value::Null,
            }]);
            player
        }),
    ];
    for (member, player) in cases {
        assert_parity(
            &with_player(player),
            false,
            &format!("an explicit null on {member}"),
        );
    }

    // The same rule governs a description on a card the player holds.
    let mut player = full_player();
    player["hand"] = json!([{
        "card_id": "card:31:Blood-Wall",
        "name": "Blood Wall",
        "cost": 2,
        "upgraded": true,
        "description": Value::Null,
    }]);
    assert_parity(
        &with_player(player),
        false,
        "an explicit null on card.description",
    );
}

#[test]
fn omitting_the_same_members_is_still_admitted() {
    // The companion to the case above: the fix must refuse only `null`, not absence. A host that
    // discloses none of this is byte-valid, which is what makes the widening additive.
    let player = full_player();
    for member in ["relics", "potions", "potion_slots", "max_potion_slots"] {
        let mut absent = player.clone();
        absent.as_object_mut().expect("an object").remove(member);
        assert_parity(&with_player(absent), true, &format!("an absent {member}"));
    }
}
