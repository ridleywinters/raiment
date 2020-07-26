#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum TileKind {
    Empty = 0,
    Grass = 1,
    Tilled = 2,
    _DebugTile = 3,
    Plants = 4,
    GrassFlowers = 5,
    Concrete = 6,
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub kind: TileKind,
    compressed_age: u8,
    packed_fields: u8,
    pub height: i16,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            kind: TileKind::Empty,
            compressed_age: 0,
            packed_fields: 0,
            height: 0,
        }
    }

    // Cache if there's a region lock on this Tile
    pub fn is_locked(&self) -> bool {
        self.packed_fields & 0x1 != 0
    }
    pub fn set_locked(&mut self, lock: bool) {
        if lock {
            self.packed_fields |= 0x1;
        } else {
            self.packed_fields &= !0x1;
        }
    }

    pub fn age(&self) -> u64 {
        (self.compressed_age as u64) * 10_000
    }
    pub fn set_age(&mut self, age: u64) {
        let m = age / 10_000;
        if m > 255 {
            self.compressed_age = 255;
        } else {
            self.compressed_age = m as u8;
        }
    }
    pub fn inc_age(&mut self) {
        if self.compressed_age < 255 {
            self.compressed_age += 1;
        }
    }
}
