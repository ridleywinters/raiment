#[derive(PartialEq, Eq, Copy, Clone, Hash)]
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

const BIT_LOCKED: u8 = 1 << 0;
const BIT_UNWALKABLE: u8 = 1 << 1;
const _BIT_UNUSED2: u8 = 1 << 2;
const _BIT_UNUSED3: u8 = 1 << 3;
const _BIT_UNUSED4: u8 = 1 << 4;
const _BIT_UNUSED5: u8 = 1 << 5;
const _BIT_UNUSED6: u8 = 1 << 6;
const _BIT_UNUSED7: u8 = 1 << 7;

impl Tile {
    pub fn new() -> Self {
        Self {
            kind: TileKind::Empty,
            compressed_age: 0,
            packed_fields: 0,
            height: 0,
        }
    }

    fn is_bit_set(&self, bit: u8) -> bool {
        self.packed_fields & bit != 0
    }

    fn set_bit(&mut self, bit: u8, value: bool) {
        if value {
            self.packed_fields |= bit;
        } else {
            self.packed_fields &= !bit;
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_bit_set(BIT_LOCKED)
    }
    pub fn set_locked(&mut self, lock: bool) {
        self.set_bit(BIT_LOCKED, lock);
    }

    pub fn is_walkable(&self) -> bool {
        !self.is_bit_set(BIT_UNWALKABLE)
    }
    pub fn set_walkable(&mut self, walkable: bool) {
        self.set_bit(BIT_UNWALKABLE, !walkable);
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
