use wasm_bindgen::prelude::*;
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

type Tile = Vec<Vec<Color>>;

#[wasm_bindgen]
pub struct WfcEngine {
    output_size: usize,
    tile_size: usize,
    tiles: Vec<Tile>,
    weights: Vec<f32>,
    adjacencies: Vec<HashMap<(isize, isize), u128>>,
    matrix: Vec<u128>, 
    entropy_map: Vec<usize>,
    rng: SmallRng,
    all_flags: u128,
    stack: Vec<(usize, usize)>,
    
    // Backtracking state
    last_contradiction_pos: Option<(usize, usize)>,
    local_reset_size: usize,
    local_reset_attempts: usize,
}

#[wasm_bindgen]
impl WfcEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(input_colors: JsValue, output_size: usize, tile_size: usize) -> Result<WfcEngine, JsValue> {
        let input: Vec<Vec<Color>> = serde_wasm_bindgen::from_value(input_colors)?;
        let rng = SmallRng::from_entropy();

        let (tiles, weights) = extract_tiles(&input, tile_size);
        if tiles.len() > 128 {
            return Err(JsValue::from_str("Too many unique patterns. Max 128."));
        }

        let all_flags = if tiles.len() == 128 {
            !0u128
        } else {
            (1u128 << tiles.len()) - 1
        };

        let adjacencies = compute_adjacencies(&tiles);

        let matrix = vec![all_flags; output_size * output_size];
        let entropy_map = vec![tiles.len(); output_size * output_size];

        Ok(WfcEngine {
            output_size,
            tile_size,
            tiles,
            weights,
            adjacencies,
            matrix,
            entropy_map,
            rng,
            all_flags,
            stack: Vec::with_capacity(output_size * output_size),
            last_contradiction_pos: None,
            local_reset_size: 8,
            local_reset_attempts: 0,
        })
    }

    pub fn step(&mut self) -> bool {
        let next_pos = self.find_lowest_entropy();
        match next_pos {
            Some(idx) => {
                let mask = self.matrix[idx];
                let chosen_tile_idx = self.observe(mask);
                self.matrix[idx] = 1 << chosen_tile_idx;
                self.entropy_map[idx] = 1;
                
                let row = idx / self.output_size;
                let col = idx % self.output_size;
                self.stack.push((row, col));
                
                if !self.propagate() {
                    self.handle_contradiction(row, col);
                    return true;
                }
                true
            }
            None => false, // Done
        }
    }

    fn handle_contradiction(&mut self, row: usize, col: usize) {
        self.local_reset_attempts += 1;
        
        if self.local_reset_attempts > 8 {
            self.local_reset_attempts = 0;
            self.local_reset_size += 4;
        }

        // If area too big, just reset everything
        if self.local_reset_size > self.output_size {
            self.reset();
        } else {
            self.reset_local(row, col, self.local_reset_size);
        }
    }

    fn reset_local(&mut self, row: usize, col: usize, size: usize) {
        let half = (size / 2) as isize;
        let r_center = row as isize;
        let c_center = col as isize;

        for dr in -half..half {
            for dc in -half..half {
                let nr = r_center + dr;
                let nc = c_center + dc;

                if nr >= 0 && nr < self.output_size as isize && nc >= 0 && nc < self.output_size as isize {
                    let idx = nr as usize * self.output_size + nc as usize;
                    self.matrix[idx] = self.all_flags;
                    self.entropy_map[idx] = self.tiles.len();
                }
            }
        }
        self.stack.clear();
        
        // After local reset, we need to re-propagate constraints from the boundary 
        // of the reset area into the reset area. For simplicity in this high-perf version,
        // we just clear the stack and let the next observe/propagate cycle handle it.
        // A more perfect backtracking would re-propagate from fixed neighbors.
    }

    fn find_lowest_entropy(&mut self) -> Option<usize> {
        let mut min_entropy = usize::MAX;
        let mut candidates = Vec::new();

        for i in 0..self.matrix.len() {
            let e = self.entropy_map[i];
            if e > 1 {
                if e < min_entropy {
                    min_entropy = e;
                    candidates.clear();
                    candidates.push(i);
                } else if e == min_entropy {
                    candidates.push(i);
                }
            }
        }

        if candidates.is_empty() {
            None
        } else {
            Some(candidates[self.rng.gen_range(0..candidates.len())])
        }
    }

    fn observe(&mut self, mask: u128) -> usize {
        let mut options = Vec::new();
        let mut total_weight = 0.0;
        for i in 0..self.tiles.len() {
            if (mask & (1 << i)) != 0 {
                options.push(i);
                total_weight += self.weights[i];
            }
        }

        if options.is_empty() {
            // This shouldn't happen if propagate works right, but safety first
            return 0;
        }

        let mut r = self.rng.gen_range(0.0..total_weight);
        for &idx in &options {
            r -= self.weights[idx];
            if r <= 0.0 {
                return idx;
            }
        }
        options[options.len() - 1]
    }

    fn propagate(&mut self) -> bool {
        while let Some((r, c)) = self.stack.pop() {
            let current_mask = self.matrix[r * self.output_size + c];

            for &(dr, dc) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nr = r as isize + dr;
                let nc = c as isize + dc;

                if nr >= 0 && nr < self.output_size as isize && nc >= 0 && nc < self.output_size as isize {
                    let nr = nr as usize;
                    let nc = nc as usize;
                    let n_idx = nr * self.output_size + nc;
                    let n_mask = self.matrix[n_idx];

                    if self.entropy_map[n_idx] <= 1 {
                        continue;
                    }

                    let mut allowed_mask = 0u128;
                    for i in 0..self.tiles.len() {
                        if (current_mask & (1 << i)) != 0 {
                            allowed_mask |= self.adjacencies[i].get(&(dr, dc)).cloned().unwrap_or(0);
                        }
                    }

                    let updated_mask = n_mask & allowed_mask;
                    if updated_mask == 0 {
                        return false; 
                    }

                    if updated_mask != n_mask {
                        self.matrix[n_idx] = updated_mask;
                        self.entropy_map[n_idx] = updated_mask.count_ones() as usize;
                        self.stack.push((nr, nc));
                    }
                }
            }
        }
        true
    }

    pub fn reset(&mut self) {
        for i in 0..self.matrix.len() {
            self.matrix[i] = self.all_flags;
            self.entropy_map[i] = self.tiles.len();
        }
        self.stack.clear();
        self.local_reset_size = 8;
        self.local_reset_attempts = 0;
    }

    pub fn get_collapsed_count(&self) -> usize {
        self.entropy_map.iter().filter(|&&e| e == 1).count()
    }

    pub fn get_image_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.output_size * self.output_size * 4);
        for &mask in &self.matrix {
            let color = self.get_display_color(mask);
            data.push(color.r);
            data.push(color.g);
            data.push(color.b);
            data.push(255);
        }
        data
    }

    fn get_display_color(&self, mask: u128) -> Color {
        let mut r = 0u32;
        let mut g = 0u32;
        let mut b = 0u32;
        let mut count = 0u32;

        for i in 0..self.tiles.len() {
            if (mask & (1 << i)) != 0 {
                let c = self.tiles[i][0][0];
                r += c.r as u32;
                g += c.g as u32;
                b += c.b as u32;
                count += 1;
            }
        }

        if count > 0 {
            Color {
                r: (r / count) as u8,
                g: (g / count) as u8,
                b: (b / count) as u8,
            }
        } else {
            Color { r: 255, g: 0, b: 255 }
        }
    }
}

fn extract_tiles(input: &Vec<Vec<Color>>, tile_size: usize) -> (Vec<Tile>, Vec<f32>) {
    let mut tile_counts: HashMap<Tile, usize> = HashMap::new();
    let rows = input.len();
    let cols = input[0].len();

    for r in 0..=(rows - tile_size) {
        for c in 0..=(cols - tile_size) {
            let mut tile = Vec::with_capacity(tile_size);
            for tr in 0..tile_size {
                let mut row = Vec::with_capacity(tile_size);
                for tc in 0..tile_size {
                    row.push(input[r + tr][c + tc]);
                }
                tile.push(row);
            }
            
            for _ in 0..4 {
                *tile_counts.entry(tile.clone()).or_insert(0) += 1;
                tile = rotate_tile(&tile);
            }
        }
    }

    let mut tiles = Vec::new();
    let mut weights = Vec::new();
    for (tile, count) in tile_counts {
        tiles.push(tile);
        weights.push(count as f32);
    }

    (tiles, weights)
}

fn rotate_tile(tile: &Tile) -> Tile {
    let size = tile.len();
    let mut new_tile = vec![vec![Color { r: 0, g: 0, b: 0 }; size]; size];
    for r in 0..size {
        for c in 0..size {
            new_tile[c][size - 1 - r] = tile[r][c];
        }
    }
    new_tile
}

fn compute_adjacencies(tiles: &Vec<Tile>) -> Vec<HashMap<(isize, isize), u128>> {
    let mut adj = vec![HashMap::new(); tiles.len()];
    for i in 0..tiles.len() {
        for j in 0..tiles.len() {
            for &(dr, dc) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                if can_overlap(&tiles[i], &tiles[j], dr, dc) {
                    *adj[i].entry((dr, dc)).or_insert(0) |= 1 << j;
                }
            }
        }
    }
    adj
}

fn can_overlap(t1: &Tile, t2: &Tile, dr: isize, dc: isize) -> bool {
    let size = t1.len() as isize;
    for r1 in 0..size {
        for c1 in 0..size {
            let r2 = r1 + dr;
            let c2 = c1 + dc;
            if r2 >= 0 && r2 < size && c2 >= 0 && c2 < size {
                if t1[r1 as usize][c1 as usize] != t2[r2 as usize][c2 as usize] {
                    return false;
                }
            }
        }
    }
    true
}
