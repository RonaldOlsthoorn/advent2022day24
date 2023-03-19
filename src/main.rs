
use std::{io::{BufReader, BufRead}, fs::File, collections::{VecDeque, HashSet, hash_map::DefaultHasher}};
use std::hash::{Hash, Hasher};

const BACKSPACE: char = 8u8 as char;


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct State {
    field: Field,
    path: Vec<Position>,
    pos: Position
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Field {
    blizzards: Vec<Blizzard>,
    rec_time: i16  
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Blizzard {
    pos: Position,
    dir: Direction
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Position {
    x: i8,
    y: i8
}

impl Position {

    const Origin: Position = Position{x: 0, y: 0};

    fn tick(self: &Self, dir: &Direction, width: &i8, height: &i8) -> Self {

        match *dir {
            Direction::Up => {
                if self.y == 0 {
                    Position{x: self.x, y: height - 1}
                } else {
                    Position{x: self.x, y: self.y - 1}
                }
            },
            Direction::Down => {
                if self.y == height - 1 {
                    Position{x: self.x, y: 0}
                } else {
                    Position{x: self.x, y: self.y + 1}
                }
            },
            Direction::Left => {
                if self.x == 0 {
                    Position{x: width - 1, y: self.y}
                } else {
                    Position{x: self.x - 1, y: self.y}
                }
            },
            Direction::Right => {
                if self.x == width - 1 {
                    Position{x: 0, y: self.y}
                } else {
                    Position{x: self.x + 1, y: self.y}
                }               
            }
        }
    }

    fn calculate_hash(self: &Self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

}

impl Hash for Position {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.x.hash(state);
        self.y.hash(state);        
    }
}

impl Blizzard {

    fn tick(self: &Self, width: &i8, height: &i8) -> Self {
        Blizzard {pos: self.pos.tick(&self.dir, width, height), dir: self.dir}
    }
}

impl Field{

    fn tick(self: &Self, width: &i8, height: &i8, period: &i16) -> Self {

        Self{blizzards: tick_blizzards(&self.blizzards, width, height), rec_time: (self.rec_time + 1).rem_euclid(*period)}
    }

    fn calculate_hash(self: &Self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for Field {

    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.rec_time.hash(state); 
    }
}

impl State {

    fn calculate_hash(self: &Self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for State {

    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.field.hash(state);
        self.pos.hash(state);
    }
}

fn tick_blizzards(field: &Vec<Blizzard>, width: &i8, height: &i8) -> Vec<Blizzard> {

    let mut res = Vec::with_capacity(field.len());

    for blizzard in field {
        res.push(blizzard.tick(width, height));
    }

    return res;
}

fn simulate(field: &Vec<Blizzard>, goal: Position, width: &i8, height: &i8) {

    let gcd = find_gcd(width, height);
    let period: i16 = (*height as i16) * ((*width as i16) / (gcd as i16));

    let mut visited_states = HashSet::new();

    let mut best_path = std::usize::MAX;

    let mut Q: VecDeque<(State, Option<Direction>, Vec<Blizzard>)> = VecDeque::new();

    let fs = find_first_states(field, width, height, &period);
    Q.extend(fs);

    while !Q.is_empty() {
        
        let (previous_state, decision, projection) = Q.pop_front().unwrap();

        //println!("");
        //print_all(&previous_state.field.blizzards, &previous_state.pos, width, height);
        //println!("decision: {:?}", decision);

        let mut current_state;
        let current_rec_time = (previous_state.field.rec_time + 1).rem_euclid(period);

        if let Some(dir) = decision {
            current_state = State{
                field: Field{blizzards: projection, rec_time: current_rec_time},
                pos: previous_state.pos.tick(&dir, width, height),
                path: previous_state.path};
            if current_state.pos == goal {

                current_state.path.push(current_state.pos);

                if current_state.path.len() < best_path {
                    println!("Found better path. Len: {}", current_state.path.len() );
                    println!("path: {:?}", current_state.path);

                    best_path = current_state.path.len();

                }
                continue;
            }
        } else {
            current_state = State{
                field: Field{blizzards: projection, rec_time: current_rec_time},
                pos: previous_state.pos,
                path: previous_state.path};
        }

        if current_state.field.blizzards.iter().any(|b| b.pos == current_state.pos) {
            println!("invalid field:");
            print_field(&current_state.field.blizzards, width, height);
            println!("pos: {:?}", &current_state.pos);
            panic!("invalid state!!")
        }

        let hash = current_state.calculate_hash();

        if visited_states.contains(&hash) {
            continue;
        }

        visited_states.insert(hash);
        current_state.path.push(current_state.pos);

        let new_projection = tick_blizzards(&current_state.field.blizzards, width, height);

        let options = determine_options(&new_projection, &current_state.pos, width, height);

        for option in options {
            Q.push_front((current_state.clone(), option.clone(), new_projection.clone()));
        }
    }
}

fn determine_options(field: &Vec<Blizzard>, pos: &Position, width: &i8, height: &i8) -> Vec<Option<Direction>> {

    let mut options = Vec::new();

    if !field.iter().any(|b| &b.pos == pos) {
        options.push(None);
    }

    let mut alt_pos = *pos;

    if pos.x == 0 {
        if pos.y == 0 {
            alt_pos.x += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Right));
            }

            alt_pos.x -= 1;
            alt_pos.y += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Down));
            }
        } else if pos.y == height - 1 {
            alt_pos.x += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Right));
            }

            alt_pos.x -= 1;
            alt_pos.y -= 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Up));
            }
        } else {
            alt_pos.y += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Down));
            }

            alt_pos.y -= 1;
            alt_pos.x += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Right));
            }

            alt_pos.y -= 1;
            alt_pos.x -= 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Up));
            }
        }
    } else if pos.x == width - 1 {
        if pos.y == 0 {
            alt_pos.x -= 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Left));
            }

            alt_pos.x += 1;
            alt_pos.y += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Down));
            }
        } else if pos.y == height - 1 {
            alt_pos.x -= 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Left));
            }

            alt_pos.x += 1;
            alt_pos.y -= 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Up));
            }
        } else {
            alt_pos.y += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Down));
            }

            alt_pos.y -= 1;
            alt_pos.x -= 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Left));
            }

            alt_pos.y -= 1;
            alt_pos.x += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Up));
            }
        }
    } else {
        alt_pos.x += 1;

        if !field.iter().any(|b| b.pos == alt_pos) {
            options.push(Some(Direction::Right));
        }

        alt_pos.x -= 2;

        if !field.iter().any(|b| b.pos == alt_pos) {
            options.push(Some(Direction::Left));
        }

        alt_pos.x += 1;

        if alt_pos.y > 0 {

            alt_pos.y -= 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Up));
            }

            alt_pos.y += 1;
        }

        if alt_pos.y < height - 1 {

            alt_pos.y += 1;

            if !field.iter().any(|b| b.pos == alt_pos) {
                options.push(Some(Direction::Down));
            }
        }
    }

    // for mid_blizzard in field.iter().filter(|b| b.pos == *pos){
    //     match mid_blizzard.dir {
    //         Direction::Up => {options.retain(|&o| if let Some(d) = o {d != Direction::Down} else {true})},
    //         Direction::Down => {options.retain(|&o| if let Some(d) = o {d != Direction::Up} else {true})},
    //         Direction::Left => {options.retain(|&o| if let Some(d) = o {d != Direction::Right} else {true})},
    //         Direction::Right => {options.retain(|&o| if let Some(d) = o {d != Direction::Left} else {true})}
    //     }    
    // }

    options
}

fn find_first_states(field: &Vec<Blizzard>, width: &i8, height: &i8, period: &i16) -> Vec<(State, Option<Direction>, Vec<Blizzard>)> {

    let start = Position{x: 0, y: -1};
    let mut first_states = Vec::new();
    let mut cache = Vec::new();
    let mut state = State{field: Field{blizzards: field.clone(), rec_time: 0}, path: vec![start], pos: start};

    while !cache.contains(&state.field) {
        
        cache.push(state.field.clone());

        let projection = tick_blizzards(&state.field.blizzards, width, height);

        if !projection.iter().any(|b| b.pos == Position::Origin) {
            first_states.push((state.clone(), Some(Direction::Down), projection.clone()));
        }

        state.field.blizzards = projection;
        state.field.rec_time = (state.field.rec_time + 1).rem_euclid(*period);
        state.path.push(start);
    }

    first_states
}

fn print_all(field: &Vec<Blizzard>, pos: &Position, width: &i8, height: &i8){

    println!("field:");
        
    for i in 0..*height {

        let mut row = String::new();

        for j in 0..*width {

            let p = Position{x: j, y: i};

            if &p == pos {
                row.push('E');
                continue;
            }

            let blizzards: Vec<Blizzard> = field.into_iter().filter(|b| if b.pos == p {true} else {false}).map(|b| *b).collect();

            if blizzards.is_empty() {
                row.push('.');
            } else if blizzards.len() == 1 {
                match blizzards[0].dir {
                    Direction::Up => {row.push('^');},
                    Direction::Down => {row.push('v');},
                    Direction::Left => {row.push('<');},
                    Direction::Right => {row.push('>');}
                }
            } else {
                row.push(char::from_digit(blizzards.len() as u32, 10).unwrap());
            }
        }

        println!("{}", row);        
    }
}

fn print_field(field: &Vec<Blizzard>, width: &i8, height: &i8) {

    println!("field:");
        
    for i in 0..*height {

        let mut row = String::new();

        for j in 0..*width {
            let p = Position{x: j, y: i};
            let blizzards: Vec<Blizzard> = field.into_iter().filter(|b| if b.pos == p {true} else {false}).map(|b| *b).collect();

            if blizzards.is_empty() {
                row.push('.');
            } else if blizzards.len() == 1 {
                match blizzards[0].dir {
                    Direction::Up => {row.push('^');},
                    Direction::Down => {row.push('v');},
                    Direction::Left => {row.push('<');},
                    Direction::Right => {row.push('>');}
                }
            } else {
                row.push(char::from_digit(blizzards.len() as u32, 10).unwrap());
            }
        }

        println!("{}", row);        
    }
}

fn find_gcd(a: &i8, b: &i8) -> i8 {

    let mut div_a = HashSet::new();

    for i in 1..*a {
        if a.rem_euclid(i) == 0 {
            div_a.insert(i);
        }
    }

    let mut div_ab = HashSet::new();

    for i in div_a {
        if b.rem_euclid(i) == 0 {
            div_ab.insert(i);
        }
    }

    div_ab.into_iter().max().unwrap()
}

fn main() {

    let lines: Vec<String> = BufReader::new(File::open("input.txt").unwrap()).lines().map(|l| l.unwrap()).collect();

    let mut field = Vec::new();

    for (row_index, line) in lines[1..lines.len() - 1].iter().enumerate() {

        field.extend(line[1..line.len() -1].chars().enumerate().filter_map(|(col_index, c)|
        if c == '^' {Some(Blizzard{pos: Position {x: col_index as i8, y: row_index as i8}, dir: Direction::Up})}
        else if c == 'v' {Some(Blizzard{pos: Position {x: col_index as i8, y: row_index as i8}, dir: Direction::Down})}
        else if c == '<' {Some(Blizzard{pos: Position {x: col_index as i8, y: row_index as i8}, dir: Direction::Left})}
        else if c == '>' {Some(Blizzard{pos: Position {x: col_index as i8, y: row_index as i8}, dir: Direction::Right})}
        else {None}));
    }

    let width = lines[0].len() as i8 - 2;
    let height = lines.len() as i8 - 2;

    simulate(&field, Position{x: width - 1, y: height - 1}, &width, &height);

    println!("completed search");
}