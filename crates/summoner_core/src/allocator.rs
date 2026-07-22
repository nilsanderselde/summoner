// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct TrackingAllocator;

thread_local! {
    static ALLOC_FORBIDDEN: Cell<bool> = const { Cell::new(false) };
}

pub struct AllocGuard;

impl AllocGuard {
    pub fn new() -> Self {
        // Warm up thread-local storage access to avoid potential allocation during first lazy init
        let _ = ALLOC_FORBIDDEN.with(|f| f.get());
        ALLOC_FORBIDDEN.with(|f| f.set(true));
        Self
    }
}

impl Default for AllocGuard {
    fn default() -> Self {
        Self::new()
    }
}


impl Drop for AllocGuard {
    fn drop(&mut self) {
        ALLOC_FORBIDDEN.with(|f| f.set(false));
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ALLOC_FORBIDDEN.with(|f| f.get()) {
            ALLOC_FORBIDDEN.with(|f| f.set(false));
            panic!("Real-time safety violation: heap allocation detected inside zero-allocation block!");
        }
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ALLOC_FORBIDDEN.with(|f| f.get()) {
            ALLOC_FORBIDDEN.with(|f| f.set(false));
            panic!("Real-time safety violation: heap deallocation detected inside zero-allocation block!");
        }
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;
