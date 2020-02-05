extern crate ddr3_simulator;
extern crate minifb;

mod xenowing;

use ddr3_simulator::*;
use minifb::{Key, Window, WindowOptions};

use xenowing::*;

use std::thread;
use std::time::Duration;

#[no_mangle]
pub extern "C" fn run(env: *const Env) -> i32 {
    let mut xenowing = Xenowing::new(env);

    let mut rom = vec![0; 0x10000];
    for (i, b) in include_bytes!("../../../rom/rom.bin").iter().enumerate() {
        rom[i] = *b;
    }

    let mut rom_addr_next = 0;

    let mut leds = 0;

    const DISPLAY_WIDTH: usize = 640;
    const DISPLAY_HEIGHT: usize = 480;
    let mut display_window = Window::new("xenowing", DISPLAY_WIDTH, DISPLAY_HEIGHT, WindowOptions::default()).expect("Couldn't open display window");
    let mut display_buffer = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
    let mut display_buffer_index = 0;
    let mut last_display_pixel_clock = 0;
    let mut last_display_vsync = 0;

    let mut time = 0;

    // Reset
    xenowing.set_reset_n(false);
    xenowing.set_clk(false);
    xenowing.eval();

    xenowing.trace_dump(time);
    time += 1;

    xenowing.set_clk(true);

    let mut ddr3_simulator = Ddr3Simulator::new();

    xenowing.set_avl_ready(ddr3_simulator.avl_ready());
    xenowing.set_avl_rdata_valid(ddr3_simulator.avl_rdata_valid());
    xenowing.set_avl_rdata(ddr3_simulator.avl_rdata());

    xenowing.eval();

    xenowing.trace_dump(time);
    time += 1;

    xenowing.set_reset_n(true);
    xenowing.set_clk(false);
    xenowing.eval();

    xenowing.trace_dump(time);
    time += 1;

    for _ in 0..2800000 {
        // Rising edge
        xenowing.set_clk(true);
        xenowing.eval();

        let rom_addr = ((rom_addr_next << 2) & 0xfffc) as usize;
        xenowing.set_program_rom_q(
            ((rom[rom_addr + 0] as u32) << 0) |
            ((rom[rom_addr + 1] as u32) << 8) |
            ((rom[rom_addr + 2] as u32) << 16) |
            ((rom[rom_addr + 3] as u32) << 24));

        rom_addr_next = xenowing.program_rom_addr();

        if xenowing.leds() != leds {
            leds = xenowing.leds();

            println!(
                "LEDS changed: {}{}{}",
                if (leds & 0x04) == 0 { 0 } else { 1 },
                if (leds & 0x02) == 0 { 0 } else { 1 },
                if (leds & 0x01) == 0 { 0 } else { 1 });
        }

        let display_pixel_clock = xenowing.display_pixel_clk();
        if display_pixel_clock == 1 && last_display_pixel_clock == 0 {
            // Pixel clock rising edge
            if xenowing.display_data_enable() == 1 {
                display_buffer[display_buffer_index] = xenowing.display_pixel_data();
                display_buffer_index += 1;
            }
        }
        last_display_pixel_clock = display_pixel_clock;

        let display_vsync = xenowing.display_vsync();
        if display_vsync == 1 && last_display_vsync == 0 {
            // Vsync rising edge
            display_window.update_with_buffer(&display_buffer).expect("Couldn't update display window");
            display_buffer_index = 0;
        }
        last_display_vsync = display_vsync;

        ddr3_simulator.set_avl_burstbegin(xenowing.avl_burstbegin());
        ddr3_simulator.set_avl_addr(xenowing.avl_addr());
        ddr3_simulator.set_avl_wdata(xenowing.avl_wdata());
        ddr3_simulator.set_avl_be(xenowing.avl_be());
        ddr3_simulator.set_avl_read_req(xenowing.avl_read_req());
        ddr3_simulator.set_avl_write_req(xenowing.avl_write_req());
        ddr3_simulator.set_avl_size(xenowing.avl_size());

        ddr3_simulator.eval();

        xenowing.set_avl_ready(ddr3_simulator.avl_ready());
        xenowing.set_avl_rdata_valid(ddr3_simulator.avl_rdata_valid());
        xenowing.set_avl_rdata(ddr3_simulator.avl_rdata());

        xenowing.eval();

        xenowing.trace_dump(time);
        time += 1;

        // Falling edge
        xenowing.set_clk(false);
        xenowing.eval();

        xenowing.trace_dump(time);
        time += 1;
    }

    xenowing.final_();

    println!("Sim done, close display window to exit");
    while display_window.is_open() && !display_window.is_key_down(Key::Escape) {
        display_window.update();
        thread::sleep(Duration::from_millis(10));
    }

    0
}
