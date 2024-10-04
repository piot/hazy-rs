/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/hazy-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use hazy_transport::{Decider, DeciderConfig, Decision};

#[test]
fn normal_decider() {
    let config = DeciderConfig {
        unaffected: 90,
        drop: 1,
        tamper: 0,
        duplicate: 3,
        reorder: 6,
    };

    let decider = Decider::new(config).expect("config should be valid");
    assert_eq!(decider.decide(89), Some(&Decision::Unaffected));
    assert_eq!(decider.decide(90), Some(&Decision::Drop));
    assert_eq!(decider.decide(91), Some(&Decision::Duplicate)); // Tamper has zero chance, and should not be in the range
}
