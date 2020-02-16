mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use rand::{Rng, SeedableRng};

use std::env;

fn main() {
    let seed = env::args().skip(1).nth(0).expect("seed not specified").parse().expect("Couldn't parse seed");
    let num_elements = env::args().skip(1).nth(1).expect("num_elements not specified").parse().expect("Couldn't parse num_elements");

    println!("Testing FIFO with seed = {} and num_elements = {}", seed, num_elements);

    let data = (0..num_elements).collect::<Vec<_>>();
    let mut data_write_ptr = 0;

    let mut read_data = Vec::new();

    let mut m = Fifo::new();
    m.reset();

    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

    let mut last_read_successful = false;
    let mut num_cycles = 0;

    loop {
        m.prop();

        // Writes
        if data_write_ptr < data.len() {
            m.write_enable = rng.gen();
            m.write_data = data[data_write_ptr];

            if m.write_enable && !m.full {
                data_write_ptr += 1;
            }
        } else {
            m.write_enable = false;
        }

        // Reads
        if last_read_successful {
            read_data.push(m.read_data);
            if read_data.len() == data.len() {
                assert_eq!(read_data, data);
                break;
            }
        }
        m.read_enable = rng.gen();
        last_read_successful = m.read_enable && !m.empty;

        m.prop();
        m.posedge_clk();
        num_cycles += 1;
    }

    println!("Test successful after {} cycles", num_cycles);
}
