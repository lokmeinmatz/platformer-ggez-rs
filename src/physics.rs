use crate::SharedWeak;
use cgmath::{InnerSpace, Point2, Vector2};
use ggez::graphics::Rect;
use itertools::Itertools;
use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub fn step_rb_sim(rbs: &mut Vec<SharedWeak<RigidBody>>, delta_time: f32) {
    let mut upgraded: Vec<_> = rbs.iter().filter_map(SharedWeak::upgrade).collect();

    for mut rb in &mut upgraded {
        let mut rb = (**rb).borrow_mut();
        if rb.weight.is_some() {
            // add gravity
            let scaled_vel = rb.velocity * delta_time;
            rb.top_left += scaled_vel;
            rb.velocity.y += delta_time;
        } else {
            rb.velocity *= 0.0;
        }
    }

    for (idx_a, idx_b) in (0..upgraded.len()).tuple_combinations::<(_, _)>() {
        let mut rb_a: RefMut<RigidBody> = (*upgraded[idx_a]).borrow_mut();
        let mut rb_b: RefMut<RigidBody> = (*upgraded[idx_b]).borrow_mut();
        if (rb_a.weight.is_some() || rb_b.weight.is_some())
            && rb_a
                .get_transformed_rect()
                .overlaps(&rb_b.get_transformed_rect())
        {
            println!("collision!");
            RigidBody::resolve_collision(&mut rb_a, &mut rb_b, cgmath::Vector2::unit_y());
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

impl RigidBody {
    pub fn resolve_collision(a: &mut RigidBody, b: &mut RigidBody, normal_a: cgmath::Vector2<f32>) {
        let relative_vel = b.velocity - a.velocity;
        let vel_normal = relative_vel.dot(normal_a);
        if vel_normal > 0.0 {
            return;
        }

        let e = a.elasticity.min(b.elasticity);

        let mut j = -(1.0 + e) * vel_normal;
        j /= 1.0 / a.weight.unwrap_or(f32::INFINITY) + 1.0 / b.weight.unwrap_or(f32::INFINITY);

        let impulse = normal_a * j;
        a.velocity -= 1.0 / a.weight.unwrap_or(f32::INFINITY) * impulse;
        b.velocity += 1.0 / b.weight.unwrap_or(f32::INFINITY) * impulse;
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
