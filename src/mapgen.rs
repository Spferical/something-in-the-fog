use std::collections::HashMap;

use bevy::prelude::*;
use rogue_algebra::Pos;

use crate::map::TileKind;

fn pos2ivec(pos: Pos) -> IVec2 {
    let Pos { x, y } = pos;
    IVec2 { x, y }
}

pub fn gen_map() -> HashMap<IVec2, TileKind> {
    let mut tile_map = rogue_algebra::TileMap::<Option<TileKind>>::new(Some(TileKind::Wall));
    // field
    let start = Pos::new(0, 0);
    for pos in rogue_algebra::Rect::new_centered(start, 5, 5).into_iter() {
        tile_map[pos] = None;
    }

    let mut map = HashMap::new();
    for (pos, tile) in tile_map.iter() {
        if let Some(tile) = tile {
            map.insert(pos2ivec(pos), tile);
        }
    }
    map
}
