use crate::world::Graphics;
use nalgebra::Translation3;

pub struct WorldEntityState {
    sync_id: u64,

    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub width: u8,
    pub length: u8,
    pub height: u8,

    color: (f32, f32, f32),
}

impl WorldEntityState {
    pub fn sync_id(&self) -> u64 {
        self.sync_id
    }

    pub fn color(&self) -> (f32, f32, f32) {
        self.color
    }

    pub fn set_color(&mut self, color: (f32, f32, f32)) {
        self.sync_id += 1;
        self.color = color;
    }
}

pub struct WorldEntity {
    pub state: WorldEntityState,
    pub graphics: Option<Graphics>,
}

impl WorldEntity {
    fn new(x: i64, y: i64, z: i64, w: i64, l: i64, h: i64) -> Self {
        let state = WorldEntityState {
            sync_id: 0,
            x,
            y,
            z,
            width: w as u8,
            length: l as u8,
            height: h as u8,
            color: (1.0, 1.0, 1.0),
        };
        Self {
            state,
            graphics: None,
        }
    }
}

pub struct WorldEntityList {
    pub entities: Vec<WorldEntity>,
}

impl WorldEntityList {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }
    pub fn add(&mut self, x: i64, y: i64, z: i64, w: i64, l: i64, h: i64, color: (f32, f32, f32)) {
        let mut entity = WorldEntity::new(x, y, z, w, l, h);
        entity.state.set_color(color);
        self.entities.push(entity);
    }
}

pub fn sync_entity(window: &mut kiss3d::window::Window, entity: &mut WorldEntity) {
    //
    // Early return if the graphics are already up to date
    //
    if let Some(graphics) = entity.graphics.as_mut() {
        if graphics.sync_id == entity.state.sync_id() {
            return;
        }
    } else {
        let node = window.add_cube(
            entity.state.width as f32,
            entity.state.height as f32,
            entity.state.length as f32,
        );
        entity.graphics = Some(Graphics {
            sync_id: entity.state.sync_id(),
            node,
        });
    }

    let graphics = entity.graphics.as_mut().unwrap();

    let t = Translation3::new(
        entity.state.x as f32 + (entity.state.width as f32) / 2.0 - 0.5,
        entity.state.z as f32 + (entity.state.height as f32) / 2.0 - 0.5,
        entity.state.y as f32 + (entity.state.length as f32) / 2.0 - 0.5,
    );
    graphics.node.set_local_translation(t);

    let (r, g, b) = entity.state.color();
    graphics.node.set_color(r, g, b);

    // graphics.sync_id = entity.state.sync_id();
}
