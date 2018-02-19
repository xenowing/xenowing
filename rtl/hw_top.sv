module hw_top(
    input reset_n,
    input clk,

    output [2:0] leds_n);

    logic [13:0] xenowing_program_rom_addr;
    logic [31:0] xenowing_program_rom_q;
    logic [2:0] xenowing_leds;
    xenowing xenowing0(
        .reset_n(reset_n),
        .clk(clk),

        .program_rom_addr(xenowing_program_rom_addr),
        .program_rom_q(xenowing_program_rom_q),

        .leds(xenowing_leds));

    program_rom program_rom0(
        .clock(clk),
        .address(xenowing_program_rom_addr),
        .q(xenowing_program_rom_q));

    assign leds_n = ~xenowing_leds;

endmodule
