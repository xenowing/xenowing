`default_nettype none

module hw_top(
    input pll_ref_clk,
    input global_reset_n,

    output [12:0] mem_a,
    output [2:0] mem_ba,
    inout mem_ck,
    inout mem_ck_n,
    output mem_cke,
    output mem_cs_n,
    output [2:0] mem_dm,
    output mem_ras_n,
    output mem_cas_n,
    output mem_we_n,
    output mem_reset_n,
    inout [23:0] mem_dq,
    inout [2:0] mem_dqs,
    inout [2:0] mem_dqs_n,
    output mem_odt,

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

    logic [2:0] xenowing_leds;
    logic xenowing_display_i2c_clk_out_n;
    logic xenowing_display_i2c_data_out_n;
    xenowing xenowing0(
        .reset_n(ddr3_controller_local_init_done && !rom_loader_system_soft_reset),
        .clk(clk),

        .program_rom_addr(program_rom_rdaddress),
        .program_rom_q(program_rom_q),

        .leds(xenowing_leds),

        .avl_ready(avl_ready),
        .avl_burstbegin(avl_burstbegin),
        .avl_addr(avl_addr),
        .avl_rdata_valid(avl_rdata_valid),
        .avl_rdata(avl_rdata[63:0]),
        .avl_wdata(avl_wdata[63:0]),
        .avl_be(avl_be[7:0]),
        .avl_read_req(avl_read_req),
        .avl_write_req(avl_write_req),
        .avl_size(avl_size),

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

    assign hdmi_scl = xenowing_display_i2c_clk_out_n ? 1'b0 : 1'bZ;
    assign hdmi_sda = xenowing_display_i2c_data_out_n ? 1'b0 : 1'bZ;

    logic [31:0] program_rom_data;
    logic [11:0] program_rom_rdaddress;
    logic [11:0] program_rom_wraddress;
    logic program_rom_wren;
    logic [31:0] program_rom_q;
    program_rom program_rom0(
        .clock(clk),
        .data(program_rom_data),
        .rdaddress(program_rom_rdaddress),
        .wraddress(program_rom_wraddress),
        .wren(program_rom_wren),
        .q(program_rom_q));

    logic clk;

    logic avl_ready;
    logic avl_burstbegin;
    logic [23:0] avl_addr;
    logic avl_rdata_valid;
    logic [95:0] avl_rdata;
    logic [95:0] avl_wdata;
    logic [11:0] avl_be;
    logic avl_read_req;
    logic avl_write_req;
    logic [6:0] avl_size;

    logic ddr3_controller_local_init_done;

    ddr3_controller ddr3_controller0(
        .pll_ref_clk(pll_ref_clk),
        .global_reset_n(global_reset_n),
        .soft_reset_n(!rom_loader_system_soft_reset),
        .afi_clk(clk),

        .mem_a(mem_a),
        .mem_ba(mem_ba),
        .mem_ck(mem_ck),
        .mem_ck_n(mem_ck_n),
        .mem_cke(mem_cke),
        .mem_cs_n(mem_cs_n),
        .mem_dm(mem_dm),
        .mem_ras_n(mem_ras_n),
        .mem_cas_n(mem_cas_n),
        .mem_we_n(mem_we_n),
        .mem_reset_n(mem_reset_n),
        .mem_dq(mem_dq),
        .mem_dqs(mem_dqs),
        .mem_dqs_n(mem_dqs_n),
        .mem_odt(mem_odt),

        .avl_ready(avl_ready),
        .avl_burstbegin(avl_burstbegin),
        .avl_addr(avl_addr),
        .avl_rdata_valid(avl_rdata_valid),
        .avl_rdata(avl_rdata),
        .avl_wdata(avl_wdata),
        .avl_be(avl_be),
        .avl_read_req(avl_read_req),
        .avl_write_req(avl_write_req),
        .avl_size(avl_size),

        .local_init_done(ddr3_controller_local_init_done));

    assign leds_n = ~xenowing_leds;

    logic [7:0] uart_receiver_data;
    logic uart_receiver_data_ready;
    uart_receiver uart_receiver0(
        .reset_n(global_reset_n),
        .clk(clk),

        .data(uart_receiver_data),
        .data_ready(uart_receiver_data_ready),

        .rx(uart_rx));

    logic rom_loader_system_soft_reset;
    rom_loader rom_loader0(
        .reset_n(global_reset_n),
        .clk(clk),

        .uart_receiver_data(uart_receiver_data),
        .uart_receiver_data_ready(uart_receiver_data_ready),

        .system_soft_reset(rom_loader_system_soft_reset),

        .program_rom_write_addr(program_rom_wraddress),
        .program_rom_write_data(program_rom_data),
        .program_rom_write_req(program_rom_wren));

endmodule
