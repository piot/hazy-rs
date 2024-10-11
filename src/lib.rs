/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/hazy-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use monotonic_time_rs::{Millis, MillisDuration};
use rand::{rngs::StdRng, Rng};
use std::cmp::Ordering;
use weighted_selector::WeightedSelector;

#[derive(Debug, PartialEq, Eq)]
pub struct Item<T> {
    pub added_at_absolute_time: Millis,
    pub absolute_time: Millis,
    pub data: T,
}

#[derive(Default)]
pub struct TimeOrderedQueue<T> {
    pub items: Vec<Item<T>>,
}

impl<T> TimeOrderedQueue<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, now: Millis, absolute_time: Millis, data: T) {
        let index = self
            .items
            .binary_search_by(|item| {
                match item.absolute_time.cmp(&absolute_time) {
                    Ordering::Equal => Ordering::Less, // if two items are inserted after eachother with the exact same absolut time, put the new item after the old one.
                    other => other,
                }
            })
            .unwrap_or_else(|x| x);

        self.items.insert(
            index,
            Item {
                added_at_absolute_time: now,
                absolute_time,
                data,
            },
        );
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn pop_ready(&mut self, absolute_time: Millis) -> Option<Item<T>> {
        let first = self.items.first()?;
        if first.absolute_time > absolute_time {
            None
        } else {
            Some(self.items.remove(0))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Drop,
    Tamper,
    Duplicate,
    Reorder,
    Unaffected,
}

pub struct Decider {
    selector: WeightedSelector<Decision>,
}

pub struct DeciderConfig {
    pub unaffected: usize,
    pub drop: usize,
    pub tamper: usize,
    pub duplicate: usize,
    pub reorder: usize,
}

impl Decider {
    pub fn new(config: DeciderConfig) -> Option<Self> {
        let selector = WeightedSelector::<Decision>::new(
            [
                (config.unaffected, Decision::Unaffected),
                (config.drop, Decision::Drop),
                (config.tamper, Decision::Tamper),
                (config.duplicate, Decision::Duplicate),
                (config.reorder, Decision::Reorder),
            ]
            .into(),
        );
        if selector.total() == 0 {
            None
        } else {
            Some(Self { selector })
        }
    }

    pub fn decide(&self, value: usize) -> Option<&Decision> {
        self.selector.select(value)
    }

    pub fn total(&self) -> usize {
        self.selector.total()
    }
}

pub struct DirectionConfig {
    pub decider: DeciderConfig,
    pub min_latency: MillisDuration,
    pub max_latency: MillisDuration,
}

pub struct Direction {
    pub decider: Decider,
    pub latency_in_ms: MillisDuration,
    pub datagrams: TimeOrderedQueue<Vec<u8>>,
    pub pseudo_random: StdRng,
}

impl Direction {
    pub fn new(config: DirectionConfig, pseudo_random: StdRng) -> Option<Self> {
        Some(Self {
            latency_in_ms: (config.min_latency + config.max_latency) / 2,
            datagrams: TimeOrderedQueue::<Vec<u8>>::new(),
            decider: Decider::new(config.decider)?,
            pseudo_random,
        })
    }

    pub fn push(&mut self, now: Millis, datagram: &[u8]) {
        let value = self.pseudo_random.gen_range(0..self.decider.total());
        let decision = self.decider.decide(value).expect("decider should not fail");
        let mut absolute_time = now + self.latency_in_ms;
        match decision {
            Decision::Drop => return,
            Decision::Tamper => todo!(),
            Decision::Duplicate => self.datagrams.push(now, absolute_time, datagram.to_vec()),
            Decision::Reorder => {
                absolute_time += (self.pseudo_random.gen_range(0..32) as u64).into();
            }
            Decision::Unaffected => {}
        }
        self.datagrams.push(now, absolute_time, datagram.to_vec());
    }

    pub fn pop_ready(&mut self, now: Millis) -> Option<Item<Vec<u8>>> {
        self.datagrams.pop_ready(now)
    }
}
