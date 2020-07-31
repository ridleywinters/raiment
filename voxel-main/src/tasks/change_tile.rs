use crate::occupation::{Context, Status2};
use crate::world::TileKind;
use Status2::*;

pub struct Builder {
    task: ChangeTileTask,
}

macro_rules! builder_field {
    ($name:ident, $field:ident, $field_type:ty) => {
        pub fn $name(mut self, $field: $field_type) -> Self {
            self.task.$field = $field;
            self
        }
    };
}

impl Builder {
    pub fn build(self) -> ChangeTileTask {
        self.task
    }

    builder_field!(with_src, src_tile_kind, Option<TileKind>);
    builder_field!(with_dst, dst_tile_kind, TileKind);
}

#[derive(PartialEq, Copy, Clone)]
pub struct ChangeTileTask {
    src_tile_kind: Option<TileKind>,
    dst_tile_kind: TileKind,
}

pub fn change_tile(dst: TileKind) -> Builder {
    Builder {
        task: ChangeTileTask {
            src_tile_kind: None,
            dst_tile_kind: dst,
        },
    }
}

impl ChangeTileTask {
    pub fn update(&mut self, ctx: &mut Context) -> Status2 {
        let (px, py) = ctx.actor_state.position();

        if let Some(src_kind) = self.src_tile_kind {
            let current_tile = ctx.map.tile(px, py);
            if current_tile.kind != src_kind {
                return Failure;
            }
        }

        ctx.map.set_kind(px, py, self.dst_tile_kind, true);
        Wait(10)
    }
}
