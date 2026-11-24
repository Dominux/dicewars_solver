use crate::rng::Rng;
use std::collections::VecDeque;

pub const GRID_W: usize = 28;
pub const GRID_H: usize = 32;
pub const GRID_SIZE: usize = GRID_W * GRID_H;
pub const MAX_TERRITORIES: usize = 32;

#[derive(Clone)]
pub struct MapData {
    pub num_territories: usize,
    /// Для каждой ячейки сетки хранит ID территории (None = вода/пустота)
    pub cells: [Option<u8>; GRID_SIZE],
    /// Битовая маска соседей для каждой территории (быстрее любого Vec!)
    pub adj_mask: [u32; MAX_TERRITORIES],
}

impl MapData {
    /// Возвращает 6 соседей для сотовой ячейки (odd-r offset coordinates)
    pub fn hex_neighbors(idx: usize) -> [Option<usize>; 6] {
        let x = (idx % GRID_W) as isize;
        let y = (idx / GRID_W) as isize;
        let is_odd = (y & 1) == 1;

        let dirs: [(isize, isize); 6] = if is_odd {
            [(1, 0), (1, -1), (0, -1), (-1, 0), (0, 1), (1, 1)]
        } else {
            [(1, 0), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1)]
        };

        let mut res = [None; 6];
        for (i, (dx, dy)) in dirs.iter().enumerate() {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < GRID_W as isize && ny >= 0 && ny < GRID_H as isize {
                res[i] = Some((ny as usize) * GRID_W + (nx as usize));
            }
        }
        res
    }

    /// Генерирует связную карту с заданным числом территорий (обычно 28..=30)
    pub fn generate(target_territories: usize, rng: &mut Rng) -> Self {
        assert!(target_territories <= MAX_TERRITORIES);

        loop {
            if let Some(map) = Self::try_generate(target_territories, rng) {
                if map.is_connected() {
                    return map;
                }
            }
        }
    }

    fn try_generate(target_territories: usize, rng: &mut Rng) -> Option<Self> {
        let mut cells = [None; GRID_SIZE];
        let mut adj_mask = [0u32; MAX_TERRITORIES];

        // Начинаем рост первой территории ближе к центру карты
        let start_cell = (GRID_H / 2) * GRID_W + (GRID_W / 2);
        let mut frontier = vec![start_cell];

        for terr_id in 0..target_territories {
            // Ищем свободную стартовую клетку, примыкающую к уже существующим территориям
            let mut seed_cell = None;
            while !frontier.is_empty() {
                let idx = rng.range(frontier.len());
                let candidate = frontier.swap_remove(idx);
                if cells[candidate].is_none() {
                    seed_cell = Some(candidate);
                    break;
                }
            }

            let start = seed_cell?;
            let target_size = rng.range_inclusive(12, 22);
            let mut region = vec![start];
            cells[start] = Some(terr_id as u8);

            // Выращиваем территорию до нужного размера
            while region.len() < target_size {
                let current = region[rng.range(region.len())];
                let mut neighbors = Self::hex_neighbors(current);
                rng.shuffle(&mut neighbors);

                let mut grew = false;
                for &n_opt in &neighbors {
                    if let Some(n) = n_opt {
                        if cells[n].is_none() {
                            cells[n] = Some(terr_id as u8);
                            region.push(n);
                            grew = true;
                            break;
                        }
                    }
                }
                if !grew {
                    // Если уперлись в тупик, проверяем, есть ли вообще свободные соседи у региона
                    let has_free = region.iter().any(|&c| {
                        Self::hex_neighbors(c)
                            .iter()
                            .flatten()
                            .any(|&n| cells[n].is_none())
                    });
                    if !has_free {
                        break;
                    }
                }
            }

            // Добавляем границы нового региона в общий фронт для следующих территорий
            for &c in &region {
                for n in Self::hex_neighbors(c).into_iter().flatten() {
                    if cells[n].is_none() {
                        frontier.push(n);
                    }
                }
            }
        }

        // Строим битовую матрицу соседства территорий
        for idx in 0..GRID_SIZE {
            if let Some(t1) = cells[idx] {
                for n in Self::hex_neighbors(idx).into_iter().flatten() {
                    if let Some(t2) = cells[n] {
                        if t1 != t2 {
                            adj_mask[t1 as usize] |= 1 << t2;
                            adj_mask[t2 as usize] |= 1 << t1;
                        }
                    }
                }
            }
        }

        Some(Self {
            num_territories: target_territories,
            cells,
            adj_mask,
        })
    }

    /// Проверяет, что все территории образуют единый связный граф (без островов)
    fn is_connected(&self) -> bool {
        let mut visited = 1u32; // Бит 0 (первая территория) посещен
        let mut queue = VecDeque::new();
        queue.push_back(0usize);

        while let Some(curr) = queue.pop_front() {
            let mut neighbors = self.adj_mask[curr];
            while neighbors != 0 {
                let next = neighbors.trailing_zeros() as usize;
                neighbors &= neighbors - 1; // Сбрасываем младший установленный бит
                if (visited & (1 << next)) == 0 {
                    visited |= 1 << next;
                    queue.push_back(next);
                }
            }
        }

        visited.count_ones() as usize == self.num_territories
    }

    /// Быстрая проверка соседства за O(1)
    #[inline]
    pub fn are_neighbors(&self, t1: u8, t2: u8) -> bool {
        (self.adj_mask[t1 as usize] & (1 << t2)) != 0
    }
}
