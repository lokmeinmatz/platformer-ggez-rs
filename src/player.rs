use crate::physics::RigidBody;
use ggez::graphics::{Mesh, DrawMode, Color, DrawParam};
use ggez::{GameResult, Context};
use cgmath::Vector2;
use crate::DebugDrawable;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Player {
    pub rb: Rc<RefCell<RigidBody>>,
    id: usize,
    sprite: Mesh
}

const RED : Color = Color::new(1.0, 0.0, 0.0, 1.0);
const GREEN : Color = Color::new(0.0, 1.0, 0.0, 1.0);
const BLUE : Color = Color::new(0.0, 0.0, 1.0, 1.0);

impl Player {
    pub fn create(ctx: &mut Context, start_pos: cgmath::Point2<f32>, id: usize) ->
                                                                                GameResult<Player> {
        let rb = RigidBody::new(start_pos, Vector2::new(1f32, 1f32), Some(1.0));
        let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rb.get_dimensions_rect(),
                                       Color::from_rgb(255, 100, 0))?;
        Ok(Player {
            rb: Rc::new(RefCell::new(rb)),
            id: 0,
            sprite: mesh
        })
    }
}

impl DebugDrawable for Player {
    fn debug_draw_worldspace(&mut self, ctx: &mut Context) -> GameResult<()> {
        let rb = self.rb.borrow();
        let bbox = Mesh::new_rectangle(ctx, DrawMode::stroke(0.1), rb.get_dimensions_rect(),
                                       GREEN)?;

        ggez::graphics::draw(ctx, &bbox, DrawParam::default().dest(rb.get_top_left()))
    }
}