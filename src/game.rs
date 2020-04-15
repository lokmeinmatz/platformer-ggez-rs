use crate::{
    utils::{
        Shared, SharedWeak, shared
    },
    world::{CellType, Tilemap},
    player::Player,
    physics::RigidBody,
    cam::Cam,
    DebugDrawable,
    physics
};

use ggez::{
    Context,
    GameResult,
    event::EventHandler,
    graphics::{self, Image, DrawParam},
    input::{
        keyboard::{KeyCode, KeyMods},
        mouse::MouseButton
    }
};

use cgmath::{Point2, Vector2, prelude::*};

use std::sync::atomic::Ordering;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Game {
    tiles: Shared<Tilemap>,
    pub cam: Cam,
    players: Vec<Shared<Player>>,
    rigidbodies: Vec<SharedWeak<RigidBody>>,
    debug_drawables: Vec<SharedWeak<dyn DebugDrawable>>,
    frame_debug_drawables: Vec<Box<dyn DebugDrawable>>
}

impl Game {
    pub fn new(ctx: &mut Context) -> Game {
        let tile_tex = Image::new(ctx, "/tiles.png").expect("No texture for tiles found");
        let mut rbs = vec![];
        let mut game = Game {
            tiles: shared(Tilemap::new(tile_tex, &mut rbs)),
            cam: Cam::new(ggez::graphics::drawable_size(ctx).into()),
            players: vec![],
            rigidbodies: rbs,
            debug_drawables: vec![],
            frame_debug_drawables: vec![]
        };

        // generate boxes
        for y in 8..=10 {
            for x in 12..20 {
                let tile_rb = game.tiles.borrow_mut().set_cell(x, y, CellType::Stone);
                if let Some(rb) = tile_rb {
                    game.rigidbodies.push(rb);
                }
            }
        }

        game.debug_drawables.push(Rc::downgrade(&game.tiles) as _);
        game.init_player(ctx, Point2::new(15.0, 1.0));
        game.init_player(ctx, Point2::new(17.0, 1.0));

        game
    }

    fn init_player(&mut self, ctx: &mut Context, pos: cgmath::Point2<f32>) -> GameResult<()> {
        let player = Rc::new(RefCell::new(Player::create(ctx, pos, 0)?));

        self.rigidbodies.push(Rc::downgrade(&player.borrow().rb));

        self.debug_drawables.push(Rc::downgrade(&player) as _);
        self.players.push(player);

        Ok(())
    }

    pub fn screen_to_world(&self, screen_pos: cgmath::Point2<f32>) -> cgmath::Point2<f32> {
        unimplemented!()
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {

        let delta = ggez::timer::delta(ctx).as_secs_f32();

        if crate::SHOULD_TERMINATE.load(Ordering::Relaxed) {
            ggez::event::quit(ctx);
            println!("Game terminated");
            return Ok(())
        }

        // Update code here...
        if ggez::timer::ticks(ctx) % 100 == 0 {
            println!("fps: {}", ggez::timer::fps(ctx));

            println!("chunks in storage: {}", self.tiles.borrow().chunks_stored());
        }

        let player_move: cgmath::Vector2<f32> = if ggez::input::keyboard::is_key_pressed(ctx, KeyCode::A) {
            (-1.0, 0.0).into()
        } else if ggez::input::keyboard::is_key_pressed(ctx, KeyCode::D) {
            (1.0, 0.0).into()
        } else { (0., 0.).into() };

        *self.players[0].borrow_mut().rb.borrow_mut().velocity_mut() += player_move * delta * 10.;

        physics::step_rb_sim(&mut self.rigidbodies, delta, &mut self.frame_debug_drawables);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let t_start = ggez::timer::time_since_start(ctx).as_secs_f32();
        //let scale = ( t_start.sin() + 6.0 ) * 8.0;
        let scale = self.cam.zoom;

        let viewport_size: cgmath::Vector2<f32> = ggez::graphics::drawable_size(ctx).into();
        let scaled_viewport_size =
            cgmath::Vector2::new(viewport_size.x / scale, viewport_size.y / scale);

        let param_scale = DrawParam::default()
            //.offset(cgmath::Point2::new(0.5, 0.5))
            .scale(cgmath::Vector2::new(scale, scale));
        let param_translate = DrawParam::default().dest(self.cam.center * -1.);
        let param_center = DrawParam::default().dest(cgmath::Point2::from_vec(viewport_size) / 2.0);
        graphics::set_transform(ctx, param_center.to_matrix());
        graphics::mul_transform(ctx, param_scale.to_matrix());
        graphics::mul_transform(ctx, param_translate.to_matrix());
        //graphics::set_transform(ctx, param.to_matrix());
        graphics::apply_transformations(ctx);

        self.tiles.borrow_mut().draw(ctx)?;

        // draw debug drawables
        for weak_drawable in &self.debug_drawables {
            if let Some(debug_draw) = weak_drawable.upgrade() {
                debug_draw.borrow_mut().debug_draw_worldspace(ctx, self)?;
            }
        }

        let mut frame_debug_drawables = vec![];
        std::mem::swap(&mut frame_debug_drawables, &mut self.frame_debug_drawables);

        for mut frame_drawable in &mut frame_debug_drawables {
            frame_drawable.debug_draw_worldspace(ctx, &self)?;
        }

        graphics::origin(ctx);

        // draw screeen space debug
        for weak_drawable in &self.debug_drawables {
            if let Some(debug_draw) = weak_drawable.upgrade() {
                debug_draw.borrow_mut().debug_draw_screenspace(ctx, self)?;
            }
        }

        for mut frame_drawable in &mut frame_debug_drawables {
            frame_drawable.debug_draw_screenspace(ctx, &self)?;
        }


        graphics::present(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
        //self.ui.update_search(key, self);
        match key {
            KeyCode::W => self.players[0].borrow_mut().jump(12.0),
            _ => {}
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        let dx = x - self.cam.last_mouse_pos.x;
        let dy = y - self.cam.last_mouse_pos.y;

        if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
            self.cam.center -= cgmath::Vector2::new(dx, dy) / self.cam.zoom;
        }

        //println!("{:?}", self.cam.center);

        self.cam.last_mouse_pos.x = x;
        self.cam.last_mouse_pos.y = y;
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        self.cam.zoom = (self.cam.zoom + y).max(4.0);
    }
}
