#![allow(dead_code)]

use crate::{Command, Entry, Result, Signal, Slot};
#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};
use arrayvec::ArrayVec;
#[cfg(feature = "chrono")]
use chrono::{DateTime, TimeZone};
use core::fmt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(bound(serialize = "C: Serialize", deserialize = "C: Deserialize<'de>"))
)]
#[derive(Clone)]
pub struct Timeline<C, F = fn(Signal)> {
    entries: ArrayVec<[Entry<C>; 32]>,
    current: usize,
    saved: Option<usize>,
    slot: Slot<F>,
}

impl<C> Timeline<C> {
    pub fn new() -> Timeline<C> {
        Builder::new().build()
    }
}

impl<C, F> Timeline<C, F> {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn limit(&self) -> usize {
        self.entries.capacity()
    }

    pub fn connect(&mut self, slot: F) -> Option<F> {
        self.slot.f.replace(slot)
    }

    pub fn disconnect(&mut self) -> Option<F> {
        self.slot.f.take()
    }

    pub fn can_undo(&self) -> bool {
        self.current() > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current() < self.len()
    }

    pub fn is_saved(&self) -> bool {
        self.saved.map_or(false, |saved| saved == self.current())
    }

    pub fn current(&self) -> usize {
        self.current
    }
}

impl<C: Command, F: FnMut(Signal)> Timeline<C, F> {
    pub fn apply(&mut self, _: &mut C::Target, _: C) -> Result<C> {
        unimplemented!()
    }

    pub fn undo(&mut self, _: &mut C::Target) -> Result<C> {
        unimplemented!()
    }

    pub fn redo(&mut self, _: &mut C::Target) -> Result<C> {
        unimplemented!()
    }

    pub fn go_to(&mut self, _: &mut C::Target, _: usize) -> Option<Result<C>> {
        unimplemented!()
    }

    #[cfg(feature = "chrono")]
    pub fn time_travel(
        &mut self,
        _: &mut C::Target,
        _: &DateTime<impl TimeZone>,
    ) -> Option<Result<C>> {
        unimplemented!()
    }

    pub fn set_saved(&mut self, saved: bool) {
        let was_saved = self.is_saved();
        if saved {
            self.saved = Some(self.current());
            self.slot.emit_if(!was_saved, Signal::Saved(true));
        } else {
            self.saved = None;
            self.slot.emit_if(was_saved, Signal::Saved(false));
        }
    }

    pub fn revert(&mut self, target: &mut C::Target) -> Option<Result<C>> {
        self.saved.and_then(|saved| self.go_to(target, saved))
    }

    pub fn clear(&mut self) {
        let could_undo = self.can_undo();
        let could_redo = self.can_redo();
        self.entries.clear();
        self.saved = if self.is_saved() { Some(0) } else { None };
        self.current = 0;
        self.slot.emit_if(could_undo, Signal::Undo(false));
        self.slot.emit_if(could_redo, Signal::Redo(false));
    }
}

#[cfg(feature = "alloc")]
impl<C: ToString, F> Timeline<C, F> {
    pub fn undo_text(&self) -> Option<String> {
        self.current.checked_sub(1).and_then(|i| self.text(i))
    }

    pub fn redo_text(&self) -> Option<String> {
        self.text(self.current)
    }

    fn text(&self, i: usize) -> Option<String> {
        self.entries.get(i).map(|e| e.command.to_string())
    }
}

impl<C> Default for Timeline<C> {
    fn default() -> Timeline<C> {
        Timeline::new()
    }
}

impl<C: fmt::Debug, F> fmt::Debug for Timeline<C, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Timeline")
            .field("entries", &self.entries)
            .field("current", &self.current)
            .field("saved", &self.saved)
            .field("slot", &self.slot)
            .finish()
    }
}

pub struct Builder<F = fn(Signal)> {
    saved: bool,
    slot: Slot<F>,
}

impl<F> Builder<F> {
    pub fn new() -> Builder<F> {
        Builder {
            saved: true,
            slot: Slot::default(),
        }
    }

    pub fn saved(mut self, saved: bool) -> Builder<F> {
        self.saved = saved;
        self
    }

    pub fn build<C>(self) -> Timeline<C, F> {
        Timeline {
            entries: ArrayVec::new(),
            current: 0,
            saved: if self.saved { Some(0) } else { None },
            slot: self.slot,
        }
    }
}

impl<F: FnMut(Signal)> Builder<F> {
    pub fn connect(mut self, f: F) -> Builder<F> {
        self.slot = Slot::from(f);
        self
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}
