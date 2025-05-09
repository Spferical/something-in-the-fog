#![allow(unused)]
use bevy::math::{IRect, IVec2};
use rand::{Rng, rngs::ThreadRng, seq::SliceRandom as _};
use rogue_algebra::{Offset, Pos, Rect, TileMap};
use std::collections::{HashMap, HashSet};

use crate::{
    map::{ItemKind, TileKind},
    mob::MobKind,
    player::GunType,
    spawn::Spawn,
};

fn get_connecting_wall(room1: Rect, room2: Rect) -> Option<Rect> {
    // one-tile-wall between them
    for (room1, room2) in &[(room1, room2), (room2, room1)] {
        // room2 right of room1
        if room1.x2 + 2 == room2.x1 {
            let y1 = room1.y1.max(room2.y1);
            let y2 = room1.y2.min(room2.y2);
            if y1 <= y2 {
                return Some(Rect {
                    x1: room1.x2 + 1,
                    x2: room1.x2 + 1,
                    y1,
                    y2,
                });
            }
        }
        // room2 under room1
        if room1.y2 + 2 == room2.y1 {
            let x1 = room1.x1.max(room2.x1);
            let x2 = room1.x2.min(room2.x2);
            if x1 <= x2 {
                return Some(Rect {
                    x1,
                    x2,
                    y1: room1.y2 + 1,
                    y2: room1.y2 + 1,
                });
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug)]
pub struct BspSplitOpts {
    max_width: i32,
    max_height: i32,
    min_width: i32,
    min_height: i32,
}

pub enum BspTree {
    Split(Box<BspTree>, Box<BspTree>),
    Room(Rect),
}

impl BspTree {
    fn into_room_graph(self) -> RoomGraph {
        match self {
            BspTree::Room(rect) => {
                let mut graph = RoomGraph::new();
                graph.add_room(rect);
                graph
            }
            BspTree::Split(tree1, tree2) => {
                let mut rooms1 = tree1.into_room_graph();
                let rooms2 = tree2.into_room_graph();
                // now figure out how to bridge the trees
                rooms1.extend_bridged(rooms2);
                rooms1
            }
        }
    }
}

struct RoomGraph {
    pub room_adj: HashMap<Rect, Vec<Rect>>,
}

impl RoomGraph {
    fn len(&self) -> usize {
        self.room_adj.len()
    }
    fn get_adj(&self, rect: Rect) -> Option<&[Rect]> {
        self.room_adj.get(&rect).map(|v| v.as_slice())
    }
    fn choose(&self, rng: &mut impl rand::Rng) -> Option<Rect> {
        if self.room_adj.is_empty() {
            return None;
        }
        let idx = rng.gen_range(0..self.room_adj.len());
        self.room_adj.keys().nth(idx).cloned()
    }
    fn find_spatially_adjacent(&'_ self, rect: Rect) -> impl Iterator<Item = Rect> + '_ {
        self.room_adj
            .keys()
            .cloned()
            .filter(move |room| get_connecting_wall(rect, *room).is_some())
    }
    fn extend_bridged(&mut self, mut other: RoomGraph) {
        let mut bridged = false;
        'loop1: for (room1, ref mut adj1) in &mut self.room_adj {
            for (room2, ref mut adj2) in &mut other.room_adj {
                if get_connecting_wall(*room1, *room2).is_some() {
                    bridged = true;
                    adj1.push(*room2);
                    adj2.push(*room1);
                    break 'loop1;
                }
            }
        }
        assert!(bridged);
        self.room_adj.extend(other.room_adj);
    }
    fn new() -> Self {
        Self {
            room_adj: HashMap::new(),
        }
    }
    fn add_room(&mut self, room: Rect) {
        self.room_adj.insert(room, vec![]);
    }
    fn add_connection(&mut self, room1: Rect, room2: Rect) {
        assert!(get_connecting_wall(room1, room2).is_some());
        assert!(self.room_adj.contains_key(&room1));
        assert!(self.room_adj.contains_key(&room2));
        self.room_adj.get_mut(&room2).unwrap().push(room1);
        self.room_adj.get_mut(&room1).unwrap().push(room2);
    }
    fn iter(&'_ self) -> impl Iterator<Item = Rect> + '_ {
        self.room_adj.keys().copied()
    }
    fn add_extra_loops(&mut self, num_loops: usize, rng: &mut impl Rng) {
        for _ in 0..num_loops {
            let room1 = self.choose(rng).unwrap();
            let room2 = self.choose(rng).unwrap();
            if get_connecting_wall(room1, room2).is_some() {
                self.add_connection(room1, room2);
            }
        }
    }
}

// returns (rooms, walls between connected rooms in the bsp tree)
pub fn gen_bsp_tree(rect: Rect, opts: BspSplitOpts, rng: &mut impl rand::Rng) -> BspTree {
    assert!(opts.min_width * 2 < opts.max_width);
    assert!(opts.min_height * 2 < opts.max_height);
    #[derive(Clone, Copy, Debug)]
    enum Split {
        X,
        Y,
        None,
    }
    let too_wide = (rect.x2 - rect.x1) > opts.max_width;
    let too_tall = (rect.y2 - rect.y1) > opts.max_height;
    let split = match (too_wide, too_tall) {
        (true, true) => *[Split::X, Split::Y].choose(rng).unwrap(),
        (true, false) => Split::X,
        (false, true) => Split::Y,
        _ => Split::None,
    };
    match split {
        Split::X => {
            let split_x = rng.gen_range(rect.x1 + opts.min_width..(rect.x2 - opts.min_width));
            let left = Rect::new(rect.x1, split_x - 1, rect.y1, rect.y2);
            let right = Rect::new(split_x + 1, rect.x2, rect.y1, rect.y2);
            BspTree::Split(
                Box::new(gen_bsp_tree(left, opts, rng)),
                Box::new(gen_bsp_tree(right, opts, rng)),
            )
        }
        Split::Y => {
            let split_y = rng.gen_range(rect.y1 + opts.min_height..(rect.y2 - opts.min_height));
            let top = Rect::new(rect.x1, rect.x2, rect.y1, split_y - 1);
            let bottom = Rect::new(rect.x1, rect.x2, split_y + 1, rect.y2);
            BspTree::Split(
                Box::new(gen_bsp_tree(top, opts, rng)),
                Box::new(gen_bsp_tree(bottom, opts, rng)),
            )
        }
        Split::None => BspTree::Room(rect),
    }
}

pub fn carve_line_drunk(
    tile_map: &mut TileMap<Option<TileKind>>,
    start: Pos,
    end: Pos,
    rng: &mut impl Rng,
    waviness: f64,
    tile: Option<TileKind>,
    bound: Rect,
) {
    let mut pos = start;
    while pos != end {
        let dir = if rng.gen_bool(waviness) {
            *rogue_algebra::CARDINALS.choose(rng).unwrap()
        } else {
            (end - pos).nearest_cardinal()
        };
        if !bound.contains(pos + dir) {
            continue;
        }
        pos += dir;
        tile_map[pos] = tile;
    }
}

pub fn gen_cellular_automata(
    room: Rect,
    iterations: usize,
    noise: f64,
    rng: &mut impl Rng,
) -> HashSet<Pos> {
    let mut state = room
        .into_iter()
        .filter(|_| rng.gen_bool(noise))
        .collect::<HashSet<Pos>>();
    for _ in 0..iterations {
        state = room
            .into_iter()
            .filter(|pos| {
                (1..=4).contains(
                    &rogue_algebra::DIRECTIONS
                        .iter()
                        .copied()
                        .filter(|dir| !state.contains(&(*pos + *dir)))
                        .count(),
                )
            })
            .collect();
    }
    state
}

pub fn dfs(starts: &[Pos], reachable: impl FnMut(Pos) -> Vec<Pos>) -> impl Iterator<Item = Pos> {
    Dfs {
        stack: starts.to_vec(),
        visited: starts.iter().cloned().collect::<HashSet<_>>(),
        reachable,
        to_emit: starts.to_vec(),
    }
}

struct Dfs<R: FnMut(Pos) -> Vec<Pos>> {
    stack: Vec<Pos>,
    visited: HashSet<Pos>,
    reachable: R,
    to_emit: Vec<Pos>,
}

impl<R: FnMut(Pos) -> Vec<Pos>> Iterator for Dfs<R> {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(p) = self.to_emit.pop() {
                return Some(p);
            }
            if let Some(p) = self.stack.pop() {
                let mut reachable = (self.reachable)(p);
                reachable.retain(|p| !self.visited.contains(p));
                self.visited.extend(reachable.iter());
                self.stack.extend(reachable.iter());
                self.to_emit.extend(reachable);
            } else {
                return None;
            }
        }
    }
}

pub fn gen_forest_room(
    tile_map: &mut TileMap<Option<TileKind>>,
    rng: &mut impl Rng,
    entrances: &[Pos],
    rect: Rect,
) {
    let mut interior_entrances = Vec::new();
    for &e in entrances {
        for &o in &rogue_algebra::CARDINALS {
            if rect.contains(e + o) {
                interior_entrances.push(e + o);
            }
        }
    }
    // draw lines between interior entrances
    for &e1 in &interior_entrances {
        for &e2 in interior_entrances.iter().chain(&[rect.center()]) {
            carve_line_drunk(tile_map, e1, e2, rng, 0.3, None, rect);
        }
    }
}

pub struct MapgenResult {
    pub spawns: HashMap<IVec2, Vec<Spawn>>,
    pub zones: Vec<IRect>,
}

pub fn get_random_empty_tile(
    tile_map: &TileMap<Option<TileKind>>,
    rect: Rect,
    rng: &mut impl Rng,
) -> Option<Pos> {
    rect.into_iter()
        .filter(|p| tile_map[*p].filter(|t| t.blocks_movement()).is_none())
        .collect::<Vec<_>>()
        .choose(rng)
        .cloned()
}

pub struct Mapgen {
    rng: ThreadRng,
    tile_map: TileMap<Option<TileKind>>,
    mob_spawns: HashMap<Pos, MobKind>,
    item_spawns: HashMap<Pos, ItemKind>,
}

impl Mapgen {
    pub fn populate(&mut self, rect: Rect, spawns: Vec<(usize, Spawn)>) {
        let total: usize = spawns.iter().map(|(c, _)| c).sum();
        let free = rect
            .into_iter()
            .filter(|p| {
                self.tile_map[*p]
                    .filter(|t| t.blocks_movement() || t.blocks_sight())
                    .is_none()
                    && !self.mob_spawns.contains_key(p)
                    && !self.item_spawns.contains_key(p)
            })
            .collect::<Vec<Pos>>();
        let mut chosen = free.choose_multiple(&mut self.rng, total);
        for (count, spawn) in spawns.into_iter() {
            for _ in 0..count {
                let pos = chosen.next().unwrap();
                match spawn {
                    Spawn::Tile(t) => {
                        self.tile_map[*pos] = Some(t);
                    }
                    Spawn::Mob(m) => {
                        self.mob_spawns.insert(*pos, m);
                    }
                    Spawn::Item(it) => {
                        self.item_spawns.insert(*pos, it);
                    }
                }
            }
        }
    }

    fn dig_rect_cellular_automata(&mut self, rect: Rect, iterations: usize, noise: f64) {
        loop {
            let walkable = gen_cellular_automata(rect, 100, 0.8, &mut self.rng);
            let starts: Vec<Pos> = rect.left_edge().into_iter().collect();
            let reachable = |p: Pos| {
                p.adjacent_cardinal()
                    .iter()
                    .cloned()
                    .filter(|p| walkable.contains(p))
                    .collect::<Vec<Pos>>()
            };
            if dfs(&starts, reachable).any(|p| rect.right_edge().contains(p)) {
                for p in dfs(&starts, reachable) {
                    self.tile_map[p] = None;
                }
                break;
            }
        }
    }
}

pub fn gen_map() -> MapgenResult {
    let mut rng = rand::thread_rng();
    let mut tile_map = TileMap::<Option<TileKind>>::new(Some(TileKind::Wall));

    let mob_spawns = HashMap::new();
    let item_spawns = HashMap::new();
    let mut mapgen = Mapgen {
        rng,
        tile_map,
        mob_spawns,
        item_spawns,
    };

    let start = Pos::new(0, 0);
    let field_rect = Rect::new_centered(start, 16, 24);
    let forest_rect = Rect::new_centered(start, 60, 24).shift_to_right_of(field_rect);
    let warehouse_zone_rect = Rect::new_centered(start, 60, 36).shift_to_right_of(forest_rect);
    let forest2_rect = Rect::new_centered(start, 60, 44).shift_to_right_of(warehouse_zone_rect);
    let railyard_rect = Rect::new_centered(start, 60, 44).shift_to_right_of(forest2_rect);
    let final_rect = Rect::new_centered(start, 20, 20).shift_to_right_of(railyard_rect);

    // field
    // field is empty and surrounded by trees on 3 sides.
    for pos in field_rect.into_iter() {
        mapgen.tile_map[pos] = if mapgen.rng.gen_bool(0.1) {
            Some(TileKind::Bush)
        } else {
            None
        }
    }
    mapgen
        .tile_map
        .set_rect(field_rect.top_edge(), Some(TileKind::Tree));
    mapgen
        .tile_map
        .set_rect(field_rect.left_edge(), Some(TileKind::Tree));
    mapgen
        .tile_map
        .set_rect(field_rect.bottom_edge(), Some(TileKind::Tree));

    // forest
    mapgen
        .tile_map
        .set_rect(forest_rect.expand_y(1), Some(TileKind::Tree));

    mapgen.dig_rect_cellular_automata(forest_rect, 100, 0.8);
    mapgen.populate(
        forest_rect,
        vec![
            (15, Spawn::Mob(MobKind::Zombie)),
            (10, Spawn::Item(ItemKind::Ammo(GunType::Pistol, 15))),
        ],
    );

    // warehouse
    mapgen
        .tile_map
        .set_rect(warehouse_zone_rect.expand_y(1), Some(TileKind::Tree));

    mapgen.tile_map.set_rect(warehouse_zone_rect, None);
    // clearing with one big building in it
    let warehouse_rect = Rect {
        x1: warehouse_zone_rect.x1 + 5,
        x2: warehouse_zone_rect.x2 - 1,
        y1: warehouse_zone_rect.y1,
        y2: warehouse_zone_rect.y2,
    };
    mapgen
        .tile_map
        .set_rect(warehouse_rect, Some(TileKind::Wall));

    let warehouse_bsp_opts = BspSplitOpts {
        min_width: 5,
        min_height: 5,
        max_width: 11,
        max_height: 11,
    };
    let warehouse_bsp_tree = gen_bsp_tree(
        warehouse_rect.shrink(1),
        warehouse_bsp_opts,
        &mut mapgen.rng,
    );
    let mut warehouse_room_graph = warehouse_bsp_tree.into_room_graph();
    // Carve out rooms, including doors between each two adjacent rooms.
    for room1 in warehouse_room_graph.iter() {
        mapgen.tile_map.set_rect(room1, None);
        // throw some crates in here
        for p in room1 {
            if mapgen.rng.gen_bool(0.02) {
                mapgen.tile_map[p] = Some(TileKind::Crate);
            }
        }
        for room2 in warehouse_room_graph.find_spatially_adjacent(room1) {
            // avoid double counting
            if room1.topleft() < room2.topleft() {
                let adj_wall = get_connecting_wall(room1, room2).unwrap();
                let door = adj_wall.choose(&mut mapgen.rng);
                mapgen.tile_map[door] = Some(TileKind::Door);
            }
        }
    }
    // add doors to the outside
    for room in warehouse_room_graph.iter() {
        let exterior_door = if room.x1 == warehouse_rect.x1 + 1 {
            Some(room.left_edge().choose(&mut mapgen.rng) + rogue_algebra::Offset::new(-1, 0))
        } else if room.x2 == warehouse_rect.x2 - 1 {
            Some(room.right_edge().choose(&mut mapgen.rng) + rogue_algebra::Offset::new(1, 0))
        } else if room.y1 == warehouse_rect.y1 + 1 {
            Some(room.bottom_edge().choose(&mut mapgen.rng) + rogue_algebra::Offset::new(0, -1))
        } else if room.y2 == warehouse_rect.y2 - 1 {
            Some(room.top_edge().choose(&mut mapgen.rng) + rogue_algebra::Offset::new(0, 1))
        } else {
            None
        };
        if let Some(door) = exterior_door {
            mapgen.tile_map[door] = Some(TileKind::Door);
        }
    }
    let sculpture_room = warehouse_room_graph.choose(&mut mapgen.rng).unwrap();
    let sculpture_room_free_spots = sculpture_room
        .into_iter()
        .filter(|p| {
            mapgen.tile_map[*p]
                .filter(|t| t.blocks_movement())
                .is_none()
        })
        .collect::<Vec<_>>();
    let [sculpture_pos, shotgun_pos] = sculpture_room_free_spots
        .choose_multiple(&mut mapgen.rng, 2)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    mapgen.mob_spawns.insert(*sculpture_pos, MobKind::Sculpture);
    mapgen.item_spawns.insert(
        *shotgun_pos,
        ItemKind::Gun(GunType::Shotgun, GunType::Shotgun.get_info().max_load),
    );
    mapgen.populate(
        warehouse_rect,
        vec![
            (20, Spawn::Mob(MobKind::Hider)),
            (7, Spawn::Item(ItemKind::Ammo(GunType::Pistol, 15))),
            (7, Spawn::Item(ItemKind::Ammo(GunType::Shotgun, 4))),
            (1, Spawn::Item(ItemKind::Gun(GunType::Shotgun, 4))),
        ],
    );

    // another forest, but with spiders and ghosts
    mapgen
        .tile_map
        .set_rect(forest2_rect.expand_y(1), Some(TileKind::Tree));
    mapgen.dig_rect_cellular_automata(forest2_rect, 100, 0.8);
    mapgen.populate(
        forest2_rect,
        vec![
            (75, Spawn::Tile(TileKind::Bush)),
            (30, Spawn::Mob(MobKind::Hider)),
            (30, Spawn::Mob(MobKind::Ghost)),
            (1, Spawn::Mob(MobKind::Sculpture)),
            (6, Spawn::Item(ItemKind::Ammo(GunType::Pistol, 15))),
            (6, Spawn::Item(ItemKind::Ammo(GunType::Shotgun, 15))),
        ],
    );

    // Railyard. Wide open but with large shipping containers obscuring vision.
    let mut boxes_zone = railyard_rect;
    boxes_zone.x1 += 1;
    boxes_zone.x2 -= 1;
    mapgen
        .tile_map
        .set_rect(boxes_zone, Some(TileKind::ShippingContainer));
    loop {
        let mut walkable = railyard_rect.into_iter().collect::<HashSet<_>>();
        for _ in 0..80 {
            let center = boxes_zone.choose(&mut mapgen.rng);
            let width = mapgen.rng.gen_range(1..=8);
            let height = mapgen.rng.gen_range(1..=8);
            let box_rect = Rect::new_centered(center, width, height)
                .intersect(&boxes_zone)
                .unwrap();
            for p in box_rect {
                walkable.remove(&p);
            }
        }
        let starts: Vec<Pos> = railyard_rect.left_edge().into_iter().collect();
        let reachable = |p: Pos| {
            p.adjacent_cardinal()
                .iter()
                .cloned()
                .filter(|p| railyard_rect.contains(*p))
                .filter(|p| walkable.contains(p))
                .collect::<Vec<Pos>>()
        };

        // flood fill to verify connectivity
        walkable = walkable
            .union(&dfs(&starts, reachable).collect::<HashSet<_>>())
            .cloned()
            .collect();
        if boxes_zone
            .right_edge()
            .into_iter()
            .any(|p| walkable.contains(&p))
        {
            for p in walkable {
                mapgen.tile_map[p] = None;
            }
            break;
        }
    }
    mapgen.populate(
        railyard_rect,
        vec![
            (11, Spawn::Mob(MobKind::Ghost)),
            (11, Spawn::Mob(MobKind::KoolAidMan)),
            (11, Spawn::Mob(MobKind::Zombie)),
            (7, Spawn::Item(ItemKind::Ammo(GunType::Pistol, 15))),
            (7, Spawn::Item(ItemKind::Ammo(GunType::Shotgun, 2))),
            (1, Spawn::Item(ItemKind::Gun(GunType::Shotgun, 2))),
        ],
    );

    // final zone: boss room
    // mapgen.tile_map.set_rect(final_rect, None);
    let bsp = gen_bsp_tree(
        final_rect,
        BspSplitOpts {
            max_width: 8,
            max_height: 8,
            min_width: 3,
            min_height: 3,
        },
        &mut mapgen.rng,
    );
    // normal bsp
    let room_graph = bsp.into_room_graph();
    for room in room_graph.iter() {
        mapgen.tile_map.set_rect(room, None);
        for adj in room_graph.get_adj(room).unwrap() {
            if room.topleft() < adj.topleft() {
                let wall = get_connecting_wall(room, *adj).unwrap();
                mapgen.tile_map[wall.choose(&mut mapgen.rng)] = Some(TileKind::Door);
            }
        }
        if room.x1 == final_rect.x1 + 1 {
            let left_door = room.left_edge().choose(&mut mapgen.rng) + Offset::new(-1, 0);
            mapgen.tile_map[left_door] = Some(TileKind::Door);
        }
    }
    // ruin it a bit
    for pos in final_rect.into_iter() {
        if mapgen.rng.gen_bool(0.2) {
            mapgen.tile_map[pos] = None;
        }
    }
    // clear out a room in the center
    let center_rect = Rect::new_centered(final_rect.center(), 8, 8);
    mapgen.tile_map.set_rect(center_rect, None);
    mapgen
        .mob_spawns
        .insert(center_rect.center(), MobKind::Eyeball);
    mapgen.populate(final_rect, vec![(10, Spawn::Mob(MobKind::Ghost))]);

    let mut spawns: HashMap<IVec2, Vec<Spawn>> = HashMap::new();
    for (pos, tile) in mapgen.tile_map.iter() {
        if let Some(tile) = tile {
            spawns
                .entry(pos.into())
                .or_default()
                .push(Spawn::Tile(tile));
        }
    }
    for (pos, mob_kind) in mapgen.mob_spawns.into_iter() {
        spawns
            .entry(pos.into())
            .or_default()
            .push(Spawn::Mob(mob_kind));
    }
    for (pos, item_kind) in mapgen.item_spawns.into_iter() {
        spawns
            .entry(pos.into())
            .or_default()
            .push(Spawn::Item(item_kind));
    }
    MapgenResult {
        spawns,
        zones: vec![
            field_rect.into(),
            forest_rect.into(),
            warehouse_zone_rect.into(),
            forest2_rect.into(),
            railyard_rect.into(),
            final_rect.into(),
        ],
    }
}
