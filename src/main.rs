
use std::{io::{BufReader, BufRead}, fs::File, collections::{VecDeque, HashSet, hash_map::DefaultHasher, HashMap}, vec, ops::Rem};
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
    rec_time: i16,
    pos: Position
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

impl State {

    fn calculate_hash(self: &Self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn tick(self: &Self, decision: Option<Direction>, fields: &HashMap<i16, Vec<Blizzard>>, width: &i8, height: &i8) -> Self {

        let next_rec_time = (self.rec_time + 1).rem_euclid(fields.len() as i16);
        let next_pos;

        if let Some(dir) = decision {
            next_pos = self.pos.tick(&dir, width, height);
        } else {
            next_pos = self.pos.clone();
        }

        Self{rec_time: next_rec_time, pos: next_pos}
    }
}

impl Hash for State {

    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.rec_time.hash(state);
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

fn manhattan(from: &Position, to: &Position) -> u16 {
    (from.x.abs_diff(to.x) + from.y.abs_diff(to.y)) as u16
}

fn simulate(fields: &HashMap<i16, Vec<Blizzard>>, width: &i8, height: &i8) -> Vec<Position> {

    let start = Position{x: 0, y: -1};
    let start_state = State{rec_time: 0, pos: start};
    let goal = Position{x: width -1, y: height - 1};

    let mut came_from: HashMap<u64, State> = HashMap::new();
    let mut open_set: HashSet<(u64, State)> = HashSet::new();
    open_set.insert((start_state.calculate_hash(), start_state.clone()));

    let mut g_score: HashMap<u64, u16> = HashMap::new();
    g_score.insert(start_state.calculate_hash(), 0);

    let mut f_score: HashMap<u64, u16> = HashMap::new();
    f_score.insert(start_state.calculate_hash(), manhattan(&start, &goal));

    while !open_set.is_empty() {

        let (current_hash, current_state) = open_set.iter().min_by(
            |(left_h, _), (right_h, _)|
            {
                let mut left = std::u16::MAX;
                if f_score.contains_key(left_h){
                    left = f_score[left_h];                
                }
                let mut  right = std::u16::MAX;
                if f_score.contains_key(right_h){
                    right = f_score[right_h];                
                }
                left.cmp(&right)
            }
        ).unwrap().clone();

        //print_all(&fields[&current_state.rec_time], &current_state.pos, width, height);

        if current_state.pos == goal {

            let mut path: Vec<Position> = vec![];
            let mut trac_back_state = current_state.clone();
            while trac_back_state.pos != start {
                path.push(trac_back_state.pos.clone());
                trac_back_state = came_from[&trac_back_state.calculate_hash()].clone();
            }

            return path.into_iter().rev().collect();
        }

        open_set.remove(&(current_hash.clone(), current_state.clone()));
        
        let neighbours = determine_options(&current_state, fields, width, height);
        let tentativ_score = g_score[&current_hash] + 1;

        for neighbour in neighbours {

            let mut neighbour_g_score = std::u16::MAX;
            let neighbour_hash = neighbour.calculate_hash();
            
            if g_score.contains_key(&neighbour_hash) {
                neighbour_g_score = g_score[&neighbour_hash];
            }

            if tentativ_score < neighbour_g_score {
                came_from.insert(neighbour_hash, current_state.clone());
                g_score.insert(neighbour_hash, tentativ_score);
                f_score.insert(neighbour_hash, tentativ_score + manhattan(&neighbour.pos, &goal));

                open_set.insert((neighbour_hash, neighbour));
            }
        }
    }

    return vec![];
}

fn determine_options(state: &State, fields: &HashMap<i16, Vec<Blizzard>>, width: &i8, height: &i8) -> Vec<State> {

    let next_rec_time = (state.rec_time + 1).rem_euclid(fields.len() as i16);
    let mut next_states = Vec::new();

    let next_field = &fields[&next_rec_time];

    let current_pos = state.pos;

    if !next_field.iter().any(|b| b.pos == current_pos) {
        next_states.push(state.tick(None, fields, width, height));
    }

    if current_pos.y == -1 {
        if !next_field.iter().any(|b| b.pos == Position{x: 0, y: 0}) {
            next_states.push(state.tick(Some(Direction::Down), fields, width, height));
        }
        return next_states;  
    }

    let mut alt_pos = current_pos;

    if current_pos.x == 0 {
        if current_pos.y == 0 {
            alt_pos.x += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Right), fields, width, height));
            }

            alt_pos.x -= 1;
            alt_pos.y += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Down), fields, width, height));
            }
        } else if current_pos.y == height - 1 {
            alt_pos.x += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Right), fields, width, height));
            }

            alt_pos.x -= 1;
            alt_pos.y -= 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Up), fields, width, height));
            }
        } else {
            alt_pos.y += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Down), fields, width, height));
            }

            alt_pos.y -= 1;
            alt_pos.x += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Right), fields, width, height));
            }

            alt_pos.y -= 1;
            alt_pos.x -= 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Up), fields, width, height));
            }
        }
    } else if current_pos.x == width - 1 {
        if current_pos.y == 0 {
            alt_pos.x -= 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Left), fields, width, height));
            }

            alt_pos.x += 1;
            alt_pos.y += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Down), fields, width, height));
            }
        } else if current_pos.y == height - 1 {
            alt_pos.x -= 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Left), fields, width, height));
            }

            alt_pos.x += 1;
            alt_pos.y -= 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Up), fields, width, height));
            }
        } else {
            alt_pos.y += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Down), fields, width, height));
            }

            alt_pos.y -= 1;
            alt_pos.x -= 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Left), fields, width, height));
            }

            alt_pos.y -= 1;
            alt_pos.x += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Up), fields, width, height));
            }
        }
    } else {
        alt_pos.x += 1;

        if !next_field.iter().any(|b| b.pos == alt_pos) {
            next_states.push(state.tick(Some(Direction::Right), fields, width, height));
        }

        alt_pos.x -= 2;

        if !next_field.iter().any(|b| b.pos == alt_pos) {
            next_states.push(state.tick(Some(Direction::Left), fields, width, height));
        }

        alt_pos.x += 1;

        if alt_pos.y > 0 {

            alt_pos.y -= 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Up), fields, width, height));
            }

            alt_pos.y += 1;
        }

        if alt_pos.y < height - 1 {

            alt_pos.y += 1;

            if !next_field.iter().any(|b| b.pos == alt_pos) {
                next_states.push(state.tick(Some(Direction::Down), fields, width, height));
            }
        }
    }

    next_states
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
   
    let mut init_field = Vec::new();

    let width;
    let height;

    {
        let lines: Vec<String> = BufReader::new(File::open("input.txt").unwrap()).lines().map(|l| l.unwrap()).collect();

        for (row_index, line) in lines[1..lines.len() - 1].iter().enumerate() {
    
            init_field.extend(line[1..line.len() -1].chars().enumerate().filter_map(|(col_index, c)|
            if c == '^' {Some(Blizzard{pos: Position {x: col_index as i8, y: row_index as i8}, dir: Direction::Up})}
            else if c == 'v' {Some(Blizzard{pos: Position {x: col_index as i8, y: row_index as i8}, dir: Direction::Down})}
            else if c == '<' {Some(Blizzard{pos: Position {x: col_index as i8, y: row_index as i8}, dir: Direction::Left})}
            else if c == '>' {Some(Blizzard{pos: Position {x: col_index as i8, y: row_index as i8}, dir: Direction::Right})}
            else {None}));
        }

        width = lines[0].len() as i8 - 2;
        height = lines.len() as i8 - 2;
    }

    let period: i16 = (height as i16) * ((width as i16) / (find_gcd(&width, &height) as i16));
    let mut fields: HashMap<i16, Vec<Blizzard>> = HashMap::new();
    
    {
        let mut f = init_field.clone();

        for i in 0..period {
            fields.insert(i, f.clone());
            f = tick_blizzards(&f, &width, &height);
        }
    }

    let path = simulate(&fields, &width, &height);

    println!("completed search. Path: {:?}", path);

}