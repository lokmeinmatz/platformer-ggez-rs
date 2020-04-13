use crate::{
    DebugDrawable,
    utils::*,
    physics::RigidBody
};
use ggez::graphics::{Color, DrawMode, DrawParam, Image, Mesh, Rect};
use ggez::{Context, GameError, GameResult};
use std::collections::HashMap;
use std::mem::MaybeUninit;

pub struct Tilemap {
    /// Maps the "chunk coords (world coords / chunk size)
    chunks: HashMap<(isize, isize), Chunk>,
    texture_atlas: Image,
}

impl Tilemap {
    pub fn new(texture_atlas: Image, rb: &mut Vec<SharedWeak<RigidBody>>) -> Self {
        let mut tm = Tilemap {
            chunks: HashMap::new(),
            texture_atlas,
        };
        tm
    }

    pub fn chunks_stored(&self) -> usize {
        self.chunks.len()
    }

    pub fn set_cell(
        &mut self,
        x: isize,
        y: isize,
        cell: CellType,
    ) -> Option<SharedWeak<RigidBody>> {
        let (cx, cy) = Chunk::to_chunk_coords(x, y);
        let cloned_tex = self.texture_atlas.clone();
        let chunk = self
            .chunks
            .entry((cx, cy))
            .or_insert_with(|| Chunk::new(cx, cy));

        chunk.set_cell(x, y, cell)
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for chunk in self.chunks.values_mut() {
            chunk.draw(ctx, &self.texture_atlas)?;
        }

        //self.chunks.get_mut(&(0, 0)).unwrap().draw(ctx, &self.texture_atlas)?;

        Ok(())
    }
}

impl DebugDrawable for Tilemap {
    fn debug_draw_worldspace(&mut self, ctx: &mut Context) -> GameResult<()> {
        let chunk_box = ggez::graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::stroke(0.1),
            Rect::new(0.0, 0.0, CHUNK_SIZE as f32, CHUNK_SIZE as f32),
            Color::from_rgb(255, 0, 0),
        )?;

        //dbg!(&chunk_box);

        for chunk in self.chunks.values() {
            ggez::graphics::draw(
                ctx,
                &chunk_box,
                DrawParam::default().dest(chunk.base_world_point()),
            )?;
        }

        Ok(())
    }
}

const CHUNK_SIZE: isize = 16;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CellType {
    Empty,
    Stone,
}

#[derive(Clone, Debug)]
struct Cell {
    cell_type: CellType,
    rb: Option<Shared<RigidBody>>,
}

impl Cell {
    pub const fn empty() -> Cell {
        Cell {
            cell_type: CellType::Empty,
            rb: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        if CellType::Empty == self.cell_type {
            true
        } else {
            false
        }
    }
}

struct Chunk {
    cells: [Cell; CHUNK_SIZE as usize * CHUNK_SIZE as usize],
    mesh_needs_update: bool,
    sprites: Option<Mesh>,
    x: isize,
    y: isize,
}

impl Chunk {
    pub fn new(x: isize, y: isize) -> Chunk {
        // needed because Cell cant implement copy because of the rb reference
        // this is valid because Cell::empty() is just static data
        let mut unsafe_cells: [MaybeUninit<Cell>; (CHUNK_SIZE * CHUNK_SIZE) as usize] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for cell in unsafe_cells.iter_mut() {
            *cell = MaybeUninit::new(Cell::empty());
        }

        Chunk {
            cells: unsafe { std::mem::transmute(unsafe_cells) },
            mesh_needs_update: true,
            sprites: None,
            x,
            y,
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, texture_atlas: &Image) -> GameResult<()> {
        /*
        let img = Image::from_rgba8(ctx, 2, 2,
                                    &[255, 0, 0, 125, 0, 255, 0, 0, 255, 255, 0, 0, 0, 255, 255,
                                        255])?;
        */
        if self.mesh_needs_update {
            self.update_mesh(ctx, texture_atlas)?;

            self.mesh_needs_update = false;
        }

        if let Some(sprites) = &self.sprites {
            ggez::graphics::draw(
                ctx,
                sprites,
                DrawParam::default().dest(cgmath::Point2::new(
                    (self.x * CHUNK_SIZE) as f32,
                    (self.y * CHUNK_SIZE) as f32,
                )),
            )?;
        }

        Ok(())
    }

    pub fn update_mesh(&mut self, ctx: &mut Context, texture_atlas: &Image) -> GameResult<()> {
        const QUAD_VERT_OFFSETS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        println!("Chunk @ ({},{}): update_mesh", self.x, self.y);
        //println!("{:?}", &self.cells);
        // check if at least one cell is not empty
        if self.cells.iter().any(|c| c.cell_type != CellType::Empty) {
            // create mesh
            let mut verts = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            for (idx, cell) in self.cells.iter().enumerate().filter(|c| !c.1.is_empty()) {
                let idx = idx as isize;
                let y = idx / CHUNK_SIZE;
                let x = idx - (y * CHUNK_SIZE);

                // create four vertices
                for [qx, qy] in QUAD_VERT_OFFSETS.iter() {
                    verts.push(ggez::graphics::Vertex {
                        pos: [*qx + x as f32, *qy + y as f32],
                        uv: [*qx * 0.25, *qy * 0.25],
                        color: [1.0; 4],
                    })
                }
                let e = verts.len() - 1;
                indices.extend(
                    [
                        e - 3,
                        e - 2,
                        e - 1, // triangle 0, 1, 2
                        e - 3,
                        e - 1,
                        e, // triangle 0, 2, 3
                    ]
                    .iter()
                    .map(|v| *v as u32),
                );
            }

            if let Some(mesh) = &mut self.sprites {
                mesh.set_vertices(ctx, &verts, &indices);
            } else {
                self.sprites = Some(Mesh::from_raw(
                    ctx,
                    &verts,
                    &indices,
                    Some(texture_atlas.clone()),
                )?);
            }
        } else {
            self.sprites = None;
        }

        Ok(())
    }

    pub fn set_cell(
        &mut self,
        x: isize,
        y: isize,
        cell_type: CellType,
    ) -> Option<SharedWeak<RigidBody>> {
        let loc_x = x - self.x * CHUNK_SIZE;
        let loc_y = y - self.y * CHUNK_SIZE;
        let cell = &mut self.cells[(loc_y * CHUNK_SIZE + loc_x) as usize];
        cell.cell_type = cell_type;

        match cell_type {
            CellType::Empty => None,
            _ => {
                let rb = shared(RigidBody::new(
                    (x as f32 - 0.05, y as f32 - 0.05).into(),
                    (1.1, 1.1).into(),
                    None,
                ));
                let ret = Some(Shared::downgrade(&rb));
                cell.rb = Some(rb);
                ret
            }
        }
    }

    fn to_chunk_coords(mut x: isize, mut y: isize) -> (isize, isize) {
        x = x - x.rem_euclid(CHUNK_SIZE);
        y = y - y.rem_euclid(CHUNK_SIZE);
        (x / CHUNK_SIZE, y / CHUNK_SIZE)
    }

    fn base_world_point(&self) -> cgmath::Point2<f32> {
        let s = CHUNK_SIZE as f32;
        cgmath::Point2::new(self.x as f32 * s, self.y as f32 * s)
    }
}
