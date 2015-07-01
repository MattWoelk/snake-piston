/* Copyright (C) 2015 by Alexandru Cojocaru */

/* This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>. */

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use std::collections::VecDeque;

//use graphics::*;
use graphics::{Context, math, color, rectangle, clear};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event::{Events, RenderEvent, PressEvent, UpdateEvent};
use piston::input::keyboard::Key;
use rand::{thread_rng, Rng};


// If you change width and height also change the levelN functions
const BOARD_WIDTH: i8 = 15;
const BOARD_HEIGHT: i8 = 15;
const TILE_SIZE: f64 = 50.0;
const UPDATE_TIME: f64 = 0.15;


fn main() {
    use glutin_window::GlutinWindow as Window;
    use piston::window::WindowSettings;

    println!("R => Restart\nP => Pause\nEsc => Quit");

    let board_size_pixels = [BOARD_WIDTH as u32 * TILE_SIZE as u32, BOARD_HEIGHT as u32 * TILE_SIZE as u32];

    let window = Window::new(
        WindowSettings::new("Snake - Piston",
                            [board_size_pixels[0], board_size_pixels[1]])
            .exit_on_esc(true));

    let mut gfx = GlGraphics::new(OpenGL::_3_2);

    let mut game = Game::new();

    for e in window.events() {
        use piston::input::Button;
        if let Some(args) = e.render_args() {
            let t = Context::new_viewport(args.viewport()).transform;
            game.render(t, &mut gfx);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            game.key_press(key);
        }

        if let Some(args) = e.update_args() {
            game.update(args.dt);
        }
    }
}


#[derive(PartialEq, Copy, Clone)]
enum State {
    Playing,
    Paused,
    GameOver,
}

#[derive(PartialEq, Copy, Clone, Debug)]
struct Point{x: i8, y: i8}

#[derive(Clone, Debug)]
struct Snake {
    tail: VecDeque<Point>,
    keys: VecDeque<Key>,
    last_pressed: Key,
}

impl Snake {
    fn new(tail: VecDeque<Point>, key: Key) -> Snake {
        Snake {
            tail: tail,
            keys: VecDeque::new(),
            last_pressed: key,
        }
    }

    fn render(&self, t: math::Matrix2d, gfx: &mut GlGraphics) {
        for p in self.tail.iter() {
            rectangle(color::hex("8ba673"),
                      rectangle::square(p.x as f64 * TILE_SIZE, p.y as f64 * TILE_SIZE, TILE_SIZE),
                      t, gfx
            );
        }
    }

    fn key_press(&mut self, k: Key) {
        use piston::input::keyboard::Key::*;
        match k {
            Right | Down | Left | Up => {
                self.keys.push_back(k);
                self.last_pressed = k;
            },
            _ => {},
        }
    }

    fn update(g: &mut Game) {
        use piston::input::keyboard::Key::*;
        if g.snakes[0].keys.is_empty() {
            let last = g.clone().snakes[0].clone().last_pressed;
            g.snakes[0].keys.push_back(last);
        }
        let k = g.snakes[0].keys.pop_front().unwrap();
        g.mv(match k {
            Right =>  Point{x: 1, y: 0},
            Down => Point{x: 0, y: 1},
            Left => Point{x: -1, y: 0},
            Up => Point{x: 0, y: -1},
            _ => panic!("only UP/DOWN/LEFT/UP arrows allowed"),
        })
    }

    fn collides(&self, xy: Point) -> bool {
        self.tail.iter().any(|t| *t == xy)
    }
}



#[derive(PartialEq, Clone)]
enum FoodType {
    Apple,
    Candy,
}

#[derive(Clone)]
struct Food {
    food_type: FoodType,
    xy: Point,
    score: u32,
    life_time: u32,
    lived_time: u32,
}

impl Food {
    fn new(t: FoodType, xy: Point, s: u32, lt: u32, probability: f64) -> Option<Food> {
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0, 100.0) < probability {
            Some(Food {
                    food_type: t,
                    xy: xy,
                    score: s,
                    life_time: lt,
                    lived_time: 0
            })
        } else {
            None
        }
    }

    fn genxy(g: &Game) -> Point {
        loop {
            let mut rng = rand::thread_rng();
            let xy = Point {x: rng.gen_range(0,BOARD_WIDTH),
                            y: rng.gen_range(0,BOARD_HEIGHT)};

            if !(g.snakes[0].tail.iter().any(|t| *t == xy) ||
                 g.food.iter().any(|f| f.xy == xy) ||
                 g.walls.iter().any(|w| *w == xy) ||
                 g.invisible_walls.iter().any(|w| *w == xy)) {
                return xy;
            }
        }
    }

    fn update(g: &mut Game) {
        if !g.food.iter().any(|f| f.food_type == FoodType::Apple) {
            if let Some(f) = Food::new(FoodType::Apple, Food::genxy(g), 10, 45, 100.0) {
                g.food.push(f)
            }
        }

        if !g.food.iter().any(|f| f.food_type == FoodType::Candy) {
            if let Some(f) = Food::new(FoodType::Candy, Food::genxy(g), 50, 15, 1.0) {
                g.food.push(f)
            }
        }

        for i in 0..g.food.len() {
            g.food[i].lived_time += 1;
            if g.food[i].lived_time > g.food[i].life_time {
                g.food.swap_remove(i);
                break;
            }
        }
    }

    fn render(&self, t: math::Matrix2d, gfx: &mut GlGraphics) {
        if self.life_time - self.lived_time < 6 && self.lived_time % 2 == 0 {
            return
        }

        let color = match self.food_type {
            FoodType::Apple => color::hex("b83e3e"),
            FoodType::Candy => color::hex("b19d46"),
        };

        rectangle(color, rectangle::square(self.xy.x as f64 * TILE_SIZE, self.xy.y as f64 * TILE_SIZE, TILE_SIZE), t, gfx);
    }
}

#[allow(unused_mut)]
macro_rules! walls {
    ( $( $x:expr, $y:expr ),* ) => {
        {
            vec![
            $(
                Point{x:$x, y:$y},
            )*
            ]
        }
    };
}

struct Level {
    snakes: Vec<Snake>,
    walls: Vec<Point>,
    invisible_walls: Vec<Point>,
}

fn level1() -> Level {

    let w = walls![
        1,0, 2,0, 3,0, 4,0, 5,0, 6,0, 8,0, 9,0, 10,0, 11,0, 12,0, 13,0,
        14,1, 14,2, 14,3, 14,4, 14,5, 14,6, 14,8, 14,9, 14,10, 14,11, 14,12, 14,13,
        1,14, 2,14, 3,14, 4,14, 5,14, 6,14, 8,14, 9,14, 10,14, 11,14, 12,14, 13,14,
        0,1, 0,2, 0,3, 0,4, 0,5, 0,6, 0,8, 0,9, 0,10, 0,11, 0,12, 0,13,
        7,7
    ];

    let iw = walls![0,0, 7,0, 14,0, 14,7, 14,14, 7,14, 0,14, 0,7];

    let mut snake1 = VecDeque::new();
    snake1.push_back(Point{x:2,y:3});
    snake1.push_back(Point{x:2,y:2});
    snake1.push_back(Point{x:2,y:1});

    let mut snake2 = VecDeque::new();
    snake2.push_back(Point{x:4,y:3});
    snake2.push_back(Point{x:4,y:2});
    snake2.push_back(Point{x:4,y:1});

    Level {
        snakes: vec![Snake::new(snake1, Key::Down),
                     Snake::new(snake2, Key::Down)],
        walls: w,
        invisible_walls: iw,
    }
}

fn level2() -> Level {
    let w = walls![
        2,2, 3,3, 4,4, 5,5, 7,7, 9,9, 10,10, 11,11, 12,12,
        12,2, 11,3, 10,4, 9,5, 7,7, 5,9, 4,10, 3,11, 2,12,
        0,7, 7,0, 14,7, 7,14
    ];

    let iw = walls![];

    let mut snake1 = VecDeque::new();
    snake1.push_back(Point{x:2,y:3});
    snake1.push_back(Point{x:2,y:2});
    snake1.push_back(Point{x:2,y:1});

    let mut snake2 = VecDeque::new();
    snake2.push_back(Point{x:4,y:3});
    snake2.push_back(Point{x:4,y:2});
    snake2.push_back(Point{x:4,y:1});

    Level {
        snakes: vec![Snake::new(snake1, Key::Down),
                     Snake::new(snake2, Key::Down)],
        walls: w,
        invisible_walls: iw,
    }
}

fn rand_level() -> Level {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0,2) {
        0 => level1(),
        1 => level2(),
        _ => panic!(""),
    }
}


#[derive(Clone)]
struct Game {
    snakes: Vec<Snake>,
    time: f64,
    update_time: f64,
    state: State,
    walls: Vec<Point>,
    invisible_walls: Vec<Point>,
    food: Vec<Food>,
    score: u32,
}

impl Game {
    fn new() -> Game {
        let l = rand_level();
        Game {snakes: l.snakes,
              time: UPDATE_TIME,
              update_time: UPDATE_TIME,
              state: State::Playing,
              walls: l.walls,
              invisible_walls: l.invisible_walls,
              food: vec![],
              score: 0,
        }
    }

    fn render(&mut self, t: math::Matrix2d, gfx: &mut GlGraphics) {
        if self.state == State::GameOver {
            clear(color::hex("000000"), gfx);
            return;
        }

        clear(color::hex("001122"), gfx);

        for ref mut f in &self.food {
            f.render(t, gfx);
        }

        for s in &self.snakes {
            s.render(t, gfx);
        }

        for w in &self.walls {
            rectangle(color::hex("002951"),
                      rectangle::square(w.x as f64 * TILE_SIZE, w.y as f64 * TILE_SIZE, TILE_SIZE),
                      t, gfx);
        }
    }

    fn update(&mut self, dt: f64) {
        match self.state {
            State::Paused | State::GameOver => return,
            _ => {},
        }

        self.time += dt;

        if self.time > self.update_time {
            self.time -= self.update_time;
            Snake::update(self);
            Food::update(self);
        }
    }

    fn key_press(&mut self, key: Key) {
        match (key, self.state) {
            (Key::R, _) => {
                let l = rand_level();
                self.snakes = l.snakes;
                self.state = State::Playing;
                self.time = UPDATE_TIME;
                self.update_time = UPDATE_TIME;
                self.walls = l.walls;
                self.invisible_walls = l.invisible_walls;
                self.food = vec![];
                self.score = 0;
                return;
            },
            (Key::P, State::Playing) => {
                self.state = State::Paused;
            },
            (Key::P, State::Paused) => {
                self.state = State::Playing;
            },
            _ => {
                self.snakes[0].key_press(key);
                self.snakes[1].key_press(key);
            }
        };
    }

    fn mv(&mut self, velocity: Point) {
        let mut i = 0;
        let mut number_dead = 0;
        for snake in &mut self.snakes {
            let next_point = next_position_with_collisions(&snake, &velocity);
            let state = calculate_game_over(&snake, &self.walls, &next_point);
            if let State::GameOver = state {
                number_dead += 1;
                continue;
            }

            for i in 0..self.food.len() {
                if self.food[i].xy == next_point {
                    let f = self.food.swap_remove(i);
                    self.score += f.score;
                    let next_tail = *snake.tail.front().unwrap();
                    snake.tail.push_back(next_tail);
                    self.update_time -= 0.002;
                    break;
                }
            }

            snake.tail.pop_back();
            snake.tail.push_front(next_point);

            i += 1;
        }

        if number_dead == 2 {
            println!("### Game Over ###\nScore: {}\nPress R to restart\nPress Esc to quit", self.score);
            self.state = State::GameOver;
        }
    }
}


fn next_position_with_collisions(snake: &Snake, velocity: &Point) -> Point{
    let mut next_point = Point{x: snake.tail.front().unwrap().x + velocity.x,
    y: snake.tail.front().unwrap().y + velocity.y};
    if next_point.x >= BOARD_WIDTH {
        next_point.x = 0;
    } else if next_point.x < 0 {
        next_point.x = BOARD_WIDTH-1;
    }

    if next_point.y >= BOARD_HEIGHT {
        next_point.y = 0;
    } else if next_point.y < 0 {
        next_point.y = BOARD_HEIGHT-1;
    }

    next_point
}

fn calculate_game_over(snake: &Snake, walls: &Vec<Point>, xy: &Point) -> State {
    if walls.iter().any(|w| *w == *xy) || snake.collides(*xy) {
        return State::GameOver;
    }

    State::Playing
}
