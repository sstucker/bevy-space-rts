use bevy::{prelude::*};

// TODO tunable in constructor
const CQT_MAX_OBJECTS_PER_NODE: usize = 60;
const CQT_MAX_LEVELS: i32 = 6;

// An entity with a position and radius
#[derive(Clone, Copy)]
pub struct EntityBody {
    pub entity: Entity,
    pub position: Vec2,
    pub radius: f32
}

// An axis-aligned rectangle defined by its top left coordinate, a width and a height.
pub struct Rectangle2D {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

pub struct CollisionQuadtree {
    pub level: i32,
    pub entities: Vec<EntityBody>,
    pub bounds: Rectangle2D,
    pub nodes: Vec<CollisionQuadtree>,
    is_split: bool
}

impl CollisionQuadtree {
    
    pub fn new(lvl: i32, bounds: Rectangle2D) -> Self {
        CollisionQuadtree {
            level: lvl,
            entities: Vec::new(),
            bounds: bounds,
            nodes: Vec::with_capacity(4),
            is_split: false
        }
    }

    pub fn clear(&mut self ) {
        self.entities.clear();
        for node in self.nodes.iter_mut() {
            node.clear();
        }
    }

    fn split(&mut self) {
        let subwidth = self.bounds.width / 2.;
        let subheight = self.bounds.height / 2.;
        let x = self.bounds.x;
        let y = self.bounds.y;
        self.nodes.push(CollisionQuadtree::new(
            self.level + 1,
            Rectangle2D {
                x: x + subwidth,
                y: y,
                width: subwidth,
                height: subheight
            }
        ));
        self.nodes.push(CollisionQuadtree::new(
            self.level + 1,
            Rectangle2D {
                x: x,
                y: y,
                width: subwidth,
                height: subheight
            }
        ));
        self.nodes.push(CollisionQuadtree::new(
            self.level + 1,
            Rectangle2D {
                x: x,
                y: y + subheight,
                width: subwidth,
                height: subheight
            }
        ));
        self.nodes.push(CollisionQuadtree::new(
            self.level + 1,
            Rectangle2D {
                x: x + subwidth,
                y: y + subheight,
                width: subwidth,
                height: subheight
            }
        ));
        self.is_split = true;
    }

    pub fn find_index(&self, p: Vec2, radius: f32) -> i32 {
        let mut index = -1;
        let midx = self.bounds.x + self.bounds.width / 2.;
        let midy = self.bounds.y + self.bounds.height / 2.;
        let top = p.y - radius > midy;
        let bottom = p.y + radius < midy;
        if p.x + radius < midx { // Left
            if top {
                index = 1;
            }
            else if bottom  {
                index = 2;
            }
        }
        else if p.x - radius > midx {
            if top {
                index = 0;
            }
            else if bottom  {
                index = 3;
            }
        }
        index
    }

    pub fn insert(&mut self, e: EntityBody) {
        if self.is_split {
            let index = self.find_index(e.position, e.radius);
            if index != -1 {
                self.nodes[index as usize].insert(e);
                return
            }
        }
        self.entities.push(e);
        if self.entities.len() > CQT_MAX_OBJECTS_PER_NODE && self.level < CQT_MAX_LEVELS {
            if !self.is_split {
                self.split();
            }
            let mut new_entities: Vec<EntityBody> = Vec::new();
            for e2 in self.entities.iter() {
                let i = self.find_index(e2.position, e2.radius);
                if i != -1 {
                    self.nodes[i as usize].insert(*e2);
                }
                else {
                    new_entities.push(*e2);
                }
            }
            self.entities = new_entities;
        }
    }
    
    pub fn retrieve(&self, p: Vec2, radius: f32, returned: &mut Vec<EntityBody>) {
        let i = self.find_index(p, radius);
        if i != -1 && self.is_split {
            self.nodes[i as usize].retrieve(p, radius, returned);
        }
        returned.extend(self.entities.iter());
    }
 
}