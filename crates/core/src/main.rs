use dicewars_core::map::{MapData, GRID_H, GRID_W};
use dicewars_core::rng::Rng;

fn main() {
    let mut rng = Rng::new(421);
    let map = MapData::generate(28, &mut rng);

    println!("Generated map with {} territories:\n", map.num_territories);
    for y in 0..GRID_H {
        if y % 2 == 1 {
            print!(" "); // Сотовый сдвиг нечетных строк
        }
        for x in 0..GRID_W {
            match map.cells[y * GRID_W + x] {
                Some(id) => print!("{:02} ", id),
                None => print!(".. "),
            }
        }
        println!();
    }
}
