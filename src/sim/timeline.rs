use super::block::{Block2D, Block3D};
use super::material::CellData;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Snapshot2D {
    pub cells: Vec<CellData>,
    pub block: Block2D,
}

#[derive(Clone)]
pub struct Snapshot3D {
    pub cells: Vec<CellData>,
    pub block: Block3D,
}

pub struct Timeline2D {
    pub snapshots: VecDeque<Snapshot2D>,
    pub cursor: usize,
    pub max_frames: usize,
}

impl Timeline2D {
    pub fn new(max_frames: usize) -> Self {
        Self {
            snapshots: VecDeque::new(),
            cursor: 0,
            max_frames,
        }
    }

    pub fn record(&mut self, cells: &[CellData], block: &Block2D) {
        if self.snapshots.len() >= self.max_frames {
            self.snapshots.pop_front();
            self.cursor = self.cursor.saturating_sub(1);
        }
        // If we were scrubbing mid-timeline, fork: drop everything after cursor
        if !self.snapshots.is_empty() && self.cursor + 1 < self.snapshots.len() {
            self.snapshots.truncate(self.cursor + 1);
        }
        self.snapshots.push_back(Snapshot2D {
            cells: cells.to_vec(),
            block: block.clone(),
        });
        self.cursor = self.snapshots.len() - 1;
    }

    pub fn scrub(&self, cursor: usize) -> Option<&Snapshot2D> {
        self.snapshots.get(cursor)
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.cursor = 0;
    }
}

pub struct Timeline3D {
    pub snapshots: VecDeque<Snapshot3D>,
    pub cursor: usize,
    pub max_frames: usize,
}

impl Timeline3D {
    pub fn new(max_frames: usize) -> Self {
        Self {
            snapshots: VecDeque::new(),
            cursor: 0,
            max_frames,
        }
    }

    pub fn record(&mut self, cells: &[CellData], block: &Block3D) {
        if self.snapshots.len() >= self.max_frames {
            self.snapshots.pop_front();
            self.cursor = self.cursor.saturating_sub(1);
        }
        if !self.snapshots.is_empty() && self.cursor + 1 < self.snapshots.len() {
            self.snapshots.truncate(self.cursor + 1);
        }
        self.snapshots.push_back(Snapshot3D {
            cells: cells.to_vec(),
            block: block.clone(),
        });
        self.cursor = self.snapshots.len() - 1;
    }

    pub fn scrub(&self, cursor: usize) -> Option<&Snapshot3D> {
        self.snapshots.get(cursor)
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.cursor = 0;
    }
}
