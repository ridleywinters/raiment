use rand::seq::SliceRandom;

use crate::occupation::{Context, Status2};
use crate::world::TileKind;
use Status2::*;

pub struct LocateTileTaskBuilder {
    task: LocateTileTask,
}

impl LocateTileTaskBuilder {
    pub fn build(self) -> LocateTileTask {
        self.task
    }

    // How many frames to delay between each random movement
    pub fn with_attempts(mut self, attempts: i8) -> LocateTileTaskBuilder {
        self.task.attempts = attempts;
        self
    }

    pub fn with_range(mut self, range: u16) -> LocateTileTaskBuilder {
        self.task.range = range;
        self
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct LocateTileTask {
    tile_kind: TileKind,
    attempts: i8,
    range: u16,

    pub destination: (i64, i64),
}

impl LocateTileTask {
    pub fn new(tile_kind: TileKind) -> LocateTileTaskBuilder {
        LocateTileTaskBuilder {
            task: LocateTileTask {
                tile_kind: tile_kind,
                attempts: 8,
                range: 72,
                destination: (0, 0),
            },
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> Status2 {
        if self.attempts <= 0 {
            return Failure;
        }

        // TODO: start with a small search range on the first attempt
        // and build towards the full range: i.e. simulate the actor
        // biasing toward looking nearby first.
        let p = ctx
            .actor_state
            .beacon_point_with_random(ctx.rng, self.range as i64);

        for dy in -3..3 {
            for dx in -3..3 {
                let q = (p.0 + dx, p.1 + dy);
                let tile = ctx.map.tile(q.0, q.1);
                if tile.kind == self.tile_kind {
                    self.destination = q;
                    return Success;
                }
            }
        }
        self.attempts -= 1;
        Wait(10)
    }
}
