use crate::{SharedWeak, DebugDrawable};
use cgmath::{InnerSpace, Point2, Vector2};
use ggez::graphics::{Rect, Color, DrawParam};
use itertools::Itertools;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use ggez::{Context, GameError, GameResult};

struct CollisionDebugDraw {
    world_pos: Point2<f32>,
    displacement: Vector2<f32>
}

impl DebugDrawable for CollisionDebugDraw {
    fn debug_draw_worldspace(&mut self, ctx: &mut Context) -> GameResult<()> {
        let arrow = ggez::graphics::Mesh::new_line(
            ctx,
            &[self.world_pos, self.world_pos + self.displacement],
            0.05,
            Color::from_rgb(100, 150, 0))?;

        ggez::graphics::draw(ctx, &arrow, DrawParam::default())
    }
}

pub fn step_rb_sim(rbs: &mut Vec<SharedWeak<RigidBody>>, delta_time: f32, frame_drawables: &mut Vec<Box<dyn DebugDrawable>>) {
    let mut upgraded: Vec<_> = rbs.iter().filter_map(SharedWeak::upgrade).collect();

    for mut rb in &mut upgraded {
        let mut rb = (**rb).borrow_mut();
        if rb.weight.is_some() {
            // add gravity
            let scaled_vel = rb.velocity * delta_time;
            rb.top_left += scaled_vel;

            rb.velocity *= 1. - delta_time * 0.5;
        // gravity
        rb.velocity.y += delta_time * 9.0;
        } else {
            rb.velocity *= 0.0;
        }
    }

    for (idx_a, idx_b) in (0..upgraded.len()).tuple_combinations::<(_, _)>() {
        let mut rb_a: RefMut<RigidBody> = (*upgraded[idx_a]).borrow_mut();
        let mut rb_b: RefMut<RigidBody> = (*upgraded[idx_b]).borrow_mut();
        if rb_a.weight.is_none() && rb_b.weight.is_none() {
            continue;
        };
        if let Some(displacement) =
            RigidBody::get_collision_displacement(rb_a.borrow(), rb_b.borrow())
        {
            //println!("tl {:?} | displ {:?}", rb_a.top_left, displacement);
            frame_drawables.push(Box::new(CollisionDebugDraw{world_pos: rb_a.top_left, displacement: displacement * 2.0}));
            RigidBody::resolve_collision(&mut rb_a, &mut rb_b, displacement);
        }
    }

    let upgraded = crate::utils::map_in_place(upgraded, |e| Rc::downgrade(&e));

    std::mem::replace(rbs, upgraded);
}

#[derive(Debug)]
pub struct RigidBody {
    top_left: Point2<f32>,
    dimensions: Vector2<f32>,
    velocity: Vector2<f32>,
    /// If `None`, the rb is solid
    weight: Option<f32>,
    elasticity: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X,
    Y,
}

impl RigidBody {
    pub fn velocity_mut(&mut self) -> &mut Vector2<f32> {
        &mut self.velocity
    }

    pub fn get_collision_displacement(
        a: &RigidBody,
        b: &RigidBody,
    ) -> Option<cgmath::Vector2<f32>> {
        let a = a.get_transformed_rect();
        let b = b.get_transformed_rect();

        if a.overlaps(&b) {
            let displ_x_ab = a.right() - b.left();
            let displ_x_ba = b.right() - a.left();

            let displ_y_ab = a.bottom() - b.top();
            let displ_y_ba = b.bottom() - a.top();

            let displ_x = if displ_x_ab > displ_x_ba {
                displ_x_ba
            } else {
                -displ_x_ab
            };
            let displ_y = if displ_y_ab > displ_y_ba {
                displ_y_ba
            } else {
                -displ_y_ab
            };
            if displ_x == 0. && displ_y == 0. {
                return None;
            }

            return Some((displ_x, displ_y).into());
        }

        None
    }

    pub fn resolve_collision(
        a: &mut RigidBody,
        b: &mut RigidBody,
        mut displace_a: cgmath::Vector2<f32>,
    ) {

        let axis_to_fix = if displace_a.x.abs() > displace_a.y.abs() {
            Axis::Y
        } else {
            Axis::X
        };

        //println!("{:?}", axis_to_fix);

        // TODO move based on weight ratio?
        match axis_to_fix {
            Axis::X => {
                a.velocity.x *= -0.5;
                b.velocity.x *= -0.5;
                displace_a.y = 0.;
            },
            Axis::Y => {
                a.velocity.y *= -0.5;
                b.velocity.y *= -0.5;
                displace_a.x = 0.;
            }
        }

        if a.weight.is_some() && b.weight.is_some() {
            // move both half the way
            displace_a *= 0.5;
            a.top_left += displace_a;
            b.top_left -= displace_a;
        }
        else if a.weight.is_some() {
            a.top_left += displace_a;
        }
        else if b.weight.is_some() {
            b.top_left -= displace_a;
        }

    }

    pub fn new(top_left: Point2<f32>, dimensions: Vector2<f32>, weight: Option<f32>) -> Self {
        RigidBody {
            top_left,
            dimensions,
            velocity: Vector2::new(0.0, 0.0),
            weight,
            elasticity: 1.0,
        }
    }

    pub fn get_dimensions_rect(&self) -> Rect {
        Rect::new(0.0, 0.0, self.dimensions.x, self.dimensions.y)
    }

    pub fn get_transformed_rect(&self) -> Rect {
        Rect::new(
            self.top_left.x,
            self.top_left.y,
            self.dimensions.x,
            self.dimensions.y,
        )
    }

    pub fn get_top_left(&self) -> cgmath::Point2<f32> {
        self.top_left
    }
}

#[cfg(test)]
mod tests {
    use crate::physics::RigidBody;
    use crate::utils::mostly_eq;

    #[test]
    fn test_collision() {
        let rb_a = RigidBody::new((0.0, 0.0).into(), (1., 1.).into(), Some(1.));
        let rb_b = RigidBody::new((0.6, 0.5).into(), (1., 1.).into(), Some(1.));

        let disp =
            RigidBody::get_collision_displacement(&rb_a, &rb_b).expect("no overlap detected");

        assert!(mostly_eq(disp.x, -0.4, 0.01));
        assert!(mostly_eq(disp.y, -0.5, 0.01));
    }
}
