`default_nettype none

module hw_top(
    input pll_ref_clk,
    input global_reset_n,

    output [2:0] leds_n,

    output uart_tx,
    input uart_rx,

    inout hdmi_scl,
    inout hdmi_sda,
    output hdmi_pixel_clk,
    output hdmi_vsync,
    output hdmi_hsync,
    output hdmi_data_enable,
    output [23:0] hdmi_pixel_data);

    logic clk;
    logic clk_pll_locked;
    clk_pll clk_pll0(
        .areset(~global_reset_n),
        .inclk0(pll_ref_clk),
        .c0(clk),
        .locked(clk_pll_locked));

    logic reset_n;
    reset_synchronizer reset_synchronizer0(
        .reset_n(global_reset_n & clk_pll_locked),
        .clk(clk),
        .reset_n_sync(reset_n));

    logic [2:0] xenowing_leds;
    logic xenowing_display_i2c_clk_out_n;
    logic xenowing_display_i2c_data_out_n;
    xenowing xenowing0(
        .reset_n(reset_n & ~rom_loader_system_soft_reset),
        .clk(clk),

        .program_rom_addr(program_rom_rdaddress),
        .program_rom_q(program_rom_q),

        .leds(xenowing_leds),

        .ram_address(ram_address),
        .ram_byteena(ram_byteena),
        .ram_data(ram_data),
        .ram_wren(ram_wren),
        .ram_q(ram_q),

        .uart_tx(uart_tx),

        .display_i2c_clk_out_n(xenowing_display_i2c_clk_out_n),
        .display_i2c_data_out_n(xenowing_display_i2c_data_out_n),
        .display_i2c_clk_in(hdmi_scl),
        .display_i2c_data_in(hdmi_sda),
        .display_pixel_clk(hdmi_pixel_clk),
        .display_vsync(hdmi_vsync),
        .display_hsync(hdmi_hsync),
        .display_data_enable(hdmi_data_enable),
        .display_pixel_data(hdmi_pixel_data));

    assign leds_n = ~xenowing_leds;

    assign hdmi_scl = xenowing_display_i2c_clk_out_n ? 1'b0 : 1'bZ;
    assign hdmi_sda = xenowing_display_i2c_data_out_n ? 1'b0 : 1'bZ;

    logic [31:0] program_rom_data;
    logic [10:0] program_rom_rdaddress;
    logic [10:0] program_rom_wraddress;
    logic program_rom_wren;
    logic [31:0] program_rom_q;
    program_rom program_rom0(
        .clock(clk),
        .data(program_rom_data),
        .rdaddress(program_rom_rdaddress),
        .wraddress(program_rom_wraddress),
        .wren(program_rom_wren),
        .q(program_rom_q));

    logic [13:0] ram_address;
    logic [7:0] ram_byteena;
    logic [63:0] ram_data;
    logic ram_wren;
    logic [63:0] ram_q;
    ram ram0(
        .address(ram_address),
        .byteena(ram_byteena),
        .clock(clk),
        .data(ram_data),
        .wren(ram_wren),
        .q(ram_q));

    logic [7:0] uart_receiver_data;
    logic uart_receiver_data_ready;
    uart_receiver uart_receiver0(
        .reset_n(reset_n),
        .clk(clk),

        .data(uart_receiver_data),
        .data_ready(uart_receiver_data_ready),

        .rx(uart_rx));

    logic rom_loader_system_soft_reset;
    rom_loader rom_loader0(
        .reset_n(reset_n),
        .clk(clk),

        .uart_receiver_data(uart_receiver_data),
        .uart_receiver_data_ready(uart_receiver_data_ready),

        .system_soft_reset(rom_loader_system_soft_reset),

        .program_rom_write_addr(program_rom_wraddress),
        .program_rom_write_data(program_rom_data),
        .program_rom_write_req(program_rom_wren));

endmodule
