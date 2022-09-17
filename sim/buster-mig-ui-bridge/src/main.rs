mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use kaze::runtime::tracing::*;
use kaze::runtime::tracing::vcd::*;

use rand::{Rng, SeedableRng};

use rtl::buster_mig_ui_bridge::*;

use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter};

fn build_trace(test_name: &'static str) -> io::Result<impl Trace> {
    let mut path = env::temp_dir();
    path.push(format!("{}.vcd", test_name));
    println!("Writing trace to {:?}", path);
    let file = File::create(path)?;
    VcdTrace::new(BufWriter::new(file), 10, TimeScaleUnit::Ns)
}

fn main() -> io::Result<()> {
    let seed = env::args().skip(1).nth(0).expect("seed not specified").parse().expect("Couldn't parse seed");
    let num_commands = env::args().skip(1).nth(1).expect("num_commands not specified").parse().expect("Couldn't parse num_commands");

    println!("Testing BusterMigUiBridge with seed = {} and num_commands = {}", seed, num_commands);

    let mut remaining_calib_cycles = 10;

    let mut data = (0..256).collect::<Vec<_>>();
    let mut expected_data = data.clone();

    let mut buster_commands_issued = 0;
    let mut ui_commands_issued = 0;

    struct UiCommand {
        ui_cmd: u32,
        addr: u32,
    }

    struct UiData {
        data: u32,
        mask: u32,
    }

    let mut next_ui_command = None;
    let mut next_ui_data = None;

    struct ReadReturn {
        value: u32,
        remaining_wait_cycles: u32,
    }

    let mut read_returns: Vec<ReadReturn> = Vec::new();

    let mut expected_read_return_values = VecDeque::new();

    let trace = build_trace("BusterMigUiBridge__fuzz")?;

    let mut m = TracingBusterMigUiBridge::new(trace)?;
    let mut time_stamp = 0;

    m.reset();

    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

    while buster_commands_issued < num_commands || ui_commands_issued < num_commands || !read_returns.is_empty() {
        // Calibration
        if remaining_calib_cycles > 0 {
            remaining_calib_cycles -= 1;
            if remaining_calib_cycles == 0 {
                m.init_calib_complete = true;
            }
        }

        // Buster command issue
        if buster_commands_issued < num_commands {
            m.bus_enable = rng.gen();
            m.bus_addr = rng.gen::<u32>() & 0xff;
            m.bus_write = rng.gen();
            m.bus_write_data = rng.gen();
            m.bus_write_byte_enable = rng.gen();
        } else {
            m.bus_enable = false;
        }

        // UI command/data acceptance conditions
        if ui_commands_issued < num_commands {
            m.app_rdy = rng.gen();
            m.app_wdf_rdy = rng.gen();
        } else {
            m.app_rdy = false;
            m.app_wdf_rdy = false;
        }

        // Return next read data, if any
        m.app_rd_data_valid = false;
        read_returns = read_returns.into_iter().filter_map(|r| {
            if r.remaining_wait_cycles > 0 {
                Some(ReadReturn {
                    value: r.value,
                    remaining_wait_cycles: r.remaining_wait_cycles - 1,
                })
            } else {
                if m.app_rd_data_valid {
                    panic!("Multiple read returns in the same cycle");
                }
                m.app_rd_data = r.value;
                m.app_rd_data_valid = true;
                None
            }
        }).collect();

        m.prop();

        // Buster command acceptance
        if m.bus_enable && m.bus_ready {
            let element = &mut expected_data[m.bus_addr as usize];
            if m.bus_write {
                let mut new_value = 0;
                for i in 0..4 {
                    new_value |= if ((m.bus_write_byte_enable >> i) & 1) == 1 {
                        m.bus_write_data
                    } else {
                        *element
                    } & (0xff << (i * 8));
                }
                *element = new_value;
            } else {
                expected_read_return_values.push_back(*element);
            }

            buster_commands_issued += 1;
        }

        // UI command acceptance
        if m.app_en && m.app_rdy {
            if next_ui_command.is_some() {
                panic!("UI command already issued");
            }
            next_ui_command = Some(UiCommand {
                ui_cmd: m.app_cmd,
                addr: m.app_addr,
            });
        }
        // UI data acceptance
        if m.app_wdf_wren && m.app_wdf_rdy {
            if next_ui_data.is_some() {
                panic!("UI data already issued");
            }
            next_ui_data = Some(UiData {
                data: m.app_wdf_data,
                mask: m.app_wdf_mask,
            });
            assert_eq!(m.app_wdf_end, true);
        }

        // Process next UI command, if any
        if let Some(command) = &next_ui_command {
            let element = &mut data[command.addr as usize];
            match command.ui_cmd {
                UI_CMD_WRITE => {
                    if let Some(data) = &next_ui_data {
                        let mut new_value = 0;
                        for i in 0..4 {
                            new_value |= if ((data.mask >> i) & 1) == 0 {
                                data.data
                            } else {
                                *element
                            } & (0xff << (i * 8));
                        }
                        *element = new_value;

                        ui_commands_issued += 1;

                        next_ui_command = None;
                        next_ui_data = None;
                    }
                }
                UI_CMD_READ => {
                    if next_ui_data.is_some() {
                        panic!("Data issued with read command");
                    }

                    read_returns.push(ReadReturn {
                        value: *element,
                        remaining_wait_cycles: 2,
                    });

                    ui_commands_issued += 1;

                    next_ui_command = None;
                }
                _ => panic!("Unrecognized UI command")
            }
        }

        // Returned data check
        if m.bus_read_data_valid {
            let expected_value = expected_read_return_values.pop_front().expect("Too many read values returned");
            assert_eq!(m.bus_read_data, expected_value);
        }

        m.prop();
        m.update_trace(time_stamp)?;

        m.posedge_clk();
        time_stamp += 1;
    }

    println!("Test successful after {} cycles", time_stamp);

    Ok(())
}
