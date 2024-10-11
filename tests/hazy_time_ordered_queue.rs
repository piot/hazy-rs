/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/hazy-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use hazy_transport::{Item, TimeOrderedQueue};

#[test]
fn same_time() {
    let mut time_ordered = TimeOrderedQueue::<i32>::new();
    time_ordered.push(0.into(), 5.into(), 10);
    time_ordered.push(0.into(), 5.into(), 11);
    assert_eq!(time_ordered.pop_ready(0.into()), None);
    assert_eq!(
        time_ordered.pop_ready(5.into()),
        Some(Item {
            added_at_absolute_time: 0.into(),
            absolute_time: 5.into(),
            data: 10
        })
    );
    assert_eq!(
        time_ordered.pop_ready(8.into()),
        Some(Item {
            added_at_absolute_time: 0.into(),
            absolute_time: 5.into(),
            data: 11
        })
    );
}

#[test]
fn insert_out_of_order() {
    let mut time_ordered = TimeOrderedQueue::<i32>::new();
    time_ordered.push(0.into(), 100.into(), 10);
    time_ordered.push(0.into(), 5.into(), 11);
    assert_eq!(time_ordered.pop_ready(0.into()), None);
    assert_eq!(
        time_ordered.pop_ready(5.into()),
        Some(Item {
            added_at_absolute_time: 0.into(),
            absolute_time: 5.into(),
            data: 11
        })
    );
    assert_eq!(
        time_ordered.pop_ready(120.into()),
        Some(Item {
            added_at_absolute_time: 0.into(),
            absolute_time: 100.into(),
            data: 10
        })
    );

    assert_eq!(time_ordered.pop_ready(120.into()), None);
}
