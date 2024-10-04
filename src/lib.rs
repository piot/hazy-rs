/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/hazy-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use log::trace;
use rand::{rngs::StdRng, Rng};
use std::cmp::Ordering;
use weighted_selector::WeightedSelector;

#[derive(Debug, PartialEq, Eq)]
pub struct Item<T> {
    pub absolute_time: u64,
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

    pub fn push(&mut self, absolute_time: u64, data: T) {
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

    pub fn pop_ready(&mut self, absolute_time: u64) -> Option<Item<T>> {
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
}

pub struct Direction {
    pub decider: Decider,
    pub latency_in_ms: u64,
    pub datagrams: TimeOrderedQueue<Vec<u8>>,
    pub pseudo_random: StdRng,
}

impl Direction {
    pub fn new(config: DirectionConfig, pseudo_random: StdRng) -> Option<Self> {
        Some(Self {
            latency_in_ms: 150,
            datagrams: TimeOrderedQueue::<Vec<u8>>::new(),
            decider: Decider::new(config.decider)?,
            pseudo_random,
        })
    }

    pub fn push(&mut self, absolute_time_now_ms: u64, datagram: &[u8]) {
        let value = self.pseudo_random.gen_range(0..self.decider.total());
        let decision = self.decider.decide(value).expect("decider should not fail");
        trace!("push: decision was {:?}", decision);
        let mut absolute_time = self.latency_in_ms + absolute_time_now_ms;
        match decision {
            Decision::Drop => return,
            Decision::Tamper => todo!(),
            Decision::Duplicate => self.datagrams.push(absolute_time, datagram.to_vec()),
            Decision::Reorder => {
                absolute_time += self.pseudo_random.gen_range(0..32) as u64;
            }
            Decision::Unaffected => {}
        }
        self.datagrams.push(absolute_time, datagram.to_vec());
    }

    pub fn pop_ready(&mut self, absolute_time_now_ms: u64) -> Option<Item<Vec<u8>>> {
        self.datagrams.pop_ready(absolute_time_now_ms)
    }
}
