mod xenowing;

use xenowing::*;

#[no_mangle]
pub extern "C" fn run(env: *const Env) -> i32 {
    let mut xenowing = Xenowing::new(env);

    let rom = include_bytes!("../../../rom/rom.bin");

    let mut rom_addr_next = 0;

    let mut leds = 0;

    let mut time = 0;

    // Reset
    xenowing.set_reset_n(true);
    xenowing.eval();
    xenowing.set_reset_n(false);
    xenowing.set_clk(false);
    xenowing.eval();

    xenowing.trace_dump(time);
    time += 1;

    xenowing.set_reset_n(true);

    for _ in 0..100 {
        // Rising edge
        xenowing.set_clk(true);
        xenowing.eval();

        let rom_addr = (rom_addr_next << 2) as usize;
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
                if (leds & 0x04) == 0 { 1 } else { 0 },
                if (leds & 0x02) == 0 { 1 } else { 0 },
                if (leds & 0x01) == 0 { 1 } else { 0 });
        }

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

    0
}
