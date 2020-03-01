mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use rand::{Rng, SeedableRng};

use std::env;

fn main() {
    let seed = env::args().skip(1).nth(0).expect("seed not specified").parse().expect("Couldn't parse seed");
    let num_elements = env::args().skip(1).nth(1).expect("num_elements not specified").parse().expect("Couldn't parse num_elements");

    println!("Testing PeekBuffer with seed = {} and num_elements = {}", seed, num_elements);

    let data = (0..num_elements).collect::<Vec<_>>();
    let mut ingress_data_ptr = 0;
    let mut last_ingress_read_successful = false;

    let mut read_data = Vec::new();

    let mut m = PeekBuffer::new();
    m.reset();

    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

    let mut num_cycles = 0;

    loop {
        m.prop();

        // Ingress return
        let ingress_ready = if ingress_data_ptr < data.len() {
            m.ingress_data = data[ingress_data_ptr];
            m.ingress_data_valid = last_ingress_read_successful;
            if last_ingress_read_successful {
                ingress_data_ptr += 1;
            }
            rng.gen()
        } else {
            false
        };

        // Egress issue
        m.egress_read_enable = rng.gen();
        m.prop();

        // Ingress return
        last_ingress_read_successful = ingress_ready && m.ingress_read_enable;

        // Egress return
        if m.egress_read_enable && m.egress_ready {
            read_data.push(m.egress_data);
            if read_data.len() == data.len() {
                assert_eq!(read_data, data);
                break;
            }
        }

        m.prop();
        m.posedge_clk();
        num_cycles += 1;
    }

    println!("Test successful after {} cycles", num_cycles);
}
