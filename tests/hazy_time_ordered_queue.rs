/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/hazy-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use hazy_transport::{Item, TimeOrderedQueue};

#[test]
fn same_time() {
    let mut time_ordered = TimeOrderedQueue::<i32>::new();
    time_ordered.push(5, 10);
    time_ordered.push(5, 11);
    assert_eq!(time_ordered.pop_ready(0), None);
    assert_eq!(
        time_ordered.pop_ready(5),
        Some(Item {
            absolute_time: 5,
            data: 10
        })
    );
    assert_eq!(
        time_ordered.pop_ready(8),
        Some(Item {
            absolute_time: 5,
            data: 11
        })
    );
}

#[test]
fn insert_out_of_order() {
    let mut time_ordered = TimeOrderedQueue::<i32>::new();
    time_ordered.push(100, 10);
    time_ordered.push(5, 11);
    assert_eq!(time_ordered.pop_ready(0), None);
    assert_eq!(
        time_ordered.pop_ready(5),
        Some(Item {
            absolute_time: 5,
            data: 11
        })
    );
    assert_eq!(
        time_ordered.pop_ready(120),
        Some(Item {
            absolute_time: 100,
            data: 10
        })
    );

    assert_eq!(time_ordered.pop_ready(120), None);
}
