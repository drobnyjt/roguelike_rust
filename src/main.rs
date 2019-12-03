use tcod::colors::*;
use tcod::console::*;
use std::cmp;
use rand::Rng;
//How to import something and alias it
use tcod::map::{FovAlgorithm, Map as FovMap};

//window size and framerate constants
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
const MAX_MONSTERS_PER_ROOM: i32 = 3;

const FOV_ALGORITHM: FovAlgorithm = FovAlgorithm::Basic;
const TORCH_RADIUS: i32 = 20;
const FOV_LIGHT_WALLS: bool = true;

const PLAYER_INDEX: usize = 0;

const COLOR_DARK_WALL: Color = Color {
    r: 0,
    g: 0,
    b: 100
};

const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};

const COLOR_DARK_GROUND: Color = Color{
    r: 50,
    g: 50,
    b: 150,
};

const COLOR_LIGHT_GROUND: Color = Color{
    r: 200,
    g: 180,
    b: 50,
};

//This struct will hold all tcod values
struct Tcod {
    root: Root,
    console: Offscreen,
    fov: FovMap,
}

//shortcut type: 2D tile array
type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

//map tile and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
    explored: bool,
}

impl Tile {
    //helper method to build empty tiles
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
        }
    }
    //helper method to build wall tiles
    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight : true,
            explored: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Item {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
    name: String,
    blocks: bool,
    alive: bool,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, name: &str, color: Color, blocks: bool) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
            name: name.into(), //what the hell
            blocks: blocks,
            alive: false,
        }
    }

    pub fn move_to(&mut self, new_x: i32, new_y: i32){
        self.x = new_x;
        self.y = new_y;
    }

    pub fn draw(&self, console: &mut dyn Console) {
        console.set_default_foreground(self.color);
        console.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

fn move_by(id: usize, dx: i32, dy:i32, game: &Game, objects: &mut Vec<Object>) {
    let (x, y) = objects[id].position();
    if !position_is_blocked(x + dx, y + dy, game, objects) {
        objects[id].move_to(x + dx, y + dy);
    }
}

//Rectangle on map
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y:i32, w:i32, h:i32) -> Self {
        return Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        };
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2)/2;
        let center_y = (self.y1 + self.y2)/2;
        return (center_x, center_y);
    }

    pub fn intersects_with_rect(&self, other: &Rect) -> bool {
        return (self.x1 <= other.x2) &&
        (self.x2 >= other.x1) &&
        (self.y1 <= other.y2) &&
        (self.y2 >= other.y1);
    }
}

fn create_room(room: Rect, map: &mut Map) {
    //coordinates of rect define walls, not room interior
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn handle_keypress(tcod: &mut Tcod, game: &Game, objects: &mut Vec<Object>) -> bool{
    //in Rust, we can import namespaces in single functions
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    //key = next keypress
    let key = tcod.root.wait_for_keypress(true);

    //movement keys using match keyword
    //similar to a dict, but runs simple code
    // .. means to ignore other fields of struct for match
    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        },

        Key { code: Escape, ..} => return true, //exit game
        Key { code: Up, .. } => move_by(PLAYER_INDEX, 0, -1, game, objects),
        Key { code: Down, .. } => move_by(PLAYER_INDEX, 0, 1, game, objects),
        Key { code: Left, .. } => move_by(PLAYER_INDEX, -1, 0, game, objects),
        Key { code: Right, .. } => move_by(PLAYER_INDEX, 1, 0, game, objects),

        Key {code: NumPad7, ..} => move_by(PLAYER_INDEX, -1, -1, game, objects),
        Key {code: NumPad8, ..} => move_by(PLAYER_INDEX, 0, -1, game, objects),
        Key {code: NumPad9, ..} => move_by(PLAYER_INDEX, 1, -1, game, objects),
        Key {code: NumPad4, ..} => move_by(PLAYER_INDEX, -1, 0, game, objects),
        Key {code: NumPad6, ..} => move_by(PLAYER_INDEX, 1, 0, game, objects),
        Key {code: NumPad1, ..} => move_by(PLAYER_INDEX, -1, 1, game, objects),
        Key {code: NumPad2, ..} => move_by(PLAYER_INDEX, 0, 1, game, objects),
        Key {code: NumPad3, ..} => move_by(PLAYER_INDEX, 1, 1, game, objects),

        _ => {}
    }

    return false;
}

fn make_map(objects: &mut Vec<Object>) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    //macro to create an empty array
    let mut rooms = vec![];
    for _ in 0..MAX_ROOMS {
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        //rooms.iter() creates an iterator
        //.any() is an iterator method that'll auto go through everything
        //this is kinda similar to list comprehension in python
        //let failed = rooms
        //    .iter()
        //    .any(|other_room| new_room.intersects_with_rect(other_room));
            //lambda x: new_room.intersects_with(x)
            // the pipes | | are a "closure" in rust - apparently a form of lambda expression!
            //Why use the closure? any TAKES a closure. Not an expression. why???

        //Honestly, this doesn't seem better than a for loop...
        let mut failed = false;

        for room in &rooms {
            if new_room.intersects_with_rect(&room) {
                failed = true;
                break;
            }
        }

        if !failed {
            create_room(new_room, &mut map);

            place_objects_in_room(new_room, objects);

            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                objects[PLAYER_INDEX].move_to(new_x, new_y)
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                //this is a random bool, apparently
                if rand::random() {
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            //add new_room to room vector
            rooms.push(new_room);
        }
    }

    return map;
}

fn make_fov_map(tcod: &mut Tcod, game: &Game) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }
}

fn render(tcod: &mut Tcod, game: &mut Game, objects: &Vec<Object>, fov_recompute: bool) {
    if fov_recompute {
        let player = &objects[0];
        tcod.fov.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGORITHM);
    }

    //render game map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);

            let wall = game.map[x as usize][y as usize].block_sight;

            let color = match(visible, wall) {
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };

            let explored = &mut game.map[x as usize][y as usize].explored;
            if visible {
                *explored = true;
            }
            if *explored {
                tcod.console.set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    //render all objects
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.console);
        }
    }

    //"blit" contents of console to root
    blit(
        &tcod.console,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0
    );
}

fn place_objects_in_room(room: Rect, objects: &mut Vec<Object>) {
    let num_monsters = rand::thread_rng().gen_range(0, MAX_MONSTERS_PER_ROOM + 1);

    for _ in 0..num_monsters {
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        //honestly, this kind of assignment is wild flow control, it's kind of amazing
        let mut monster = if rand::random::<f32>() < 0.8 {
            Object::new(x, y, 'o', "orc", DESATURATED_GREEN, true)
        } else {
            Object::new(x, y, 'T', "troll", DARKER_GREEN, true)
        };

        objects.push(monster);
    }
}

fn position_is_blocked(x: i32, y: i32, game: &Game, objects: &Vec<Object>) -> bool {
    if game.map[x as usize][y as usize].blocked {
        return true;
    }

    for object in objects {
        if object.position() == (x, y) {
            return true;
        }
    }
    false
}

fn main() {
    //set up tcod console window
    //let to create new variable named root
    let root: Root = Root::initializer()
    .font("arial10x10.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("Rust/libtocd tutorial")
    .init();

    let console = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let fov = FovMap::new(MAP_WIDTH, MAP_HEIGHT);

    //mut enables mutability
    //create tcod, of type Tcod, with Root root
    let mut tcod = Tcod{ root, console, fov };

    let player = Object::new(23, 17, '@', "me", WHITE, true);
    //objects now OWNS player and npc - can't use those variable names elsewhere
    let mut objects = vec![player];

    let mut game = Game {
        map: make_map(&mut objects),
    };

    make_fov_map(&mut tcod, &game);
    let mut previous_player_position = (-1, -1);

    //Limit realtime fps to 20
    tcod::system::set_fps(LIMIT_FPS);

    //game loop
    //This game loop will run max 20 FPS because of previous tcod option
    while !tcod.root.window_closed() {

        //clear window
        tcod.console.clear();

        //compute field-of-view
        let fov_recompute = previous_player_position != (objects[PLAYER_INDEX].x, objects[PLAYER_INDEX].y);
        //render game map and all game objects
        render(&mut tcod, &mut game, &objects, fov_recompute);

        //flush buffer to screen
        tcod.root.flush();

        //game logic
        let player = &mut objects[PLAYER_INDEX];
        previous_player_position = (player.x, player.y);
        let exit = handle_keypress(&mut tcod, &game, &mut objects);
        if exit {
            break;
        }

    }
    //Finalize
    println!("Goodbye!");
}
