`default_nettype none

module xenowing(
    input reset_n,
    input clk,

    output [11:0] program_rom_addr,
    input [31:0] program_rom_q,

    output [2:0] leds,

    output uart_tx,

    input avl_ready,
    output avl_burstbegin,
    output [23:0] avl_addr,
    input avl_rdata_valid,
    input [63:0] avl_rdata,
    output [63:0] avl_wdata,
    output [7:0] avl_be,
    output avl_read_req,
    output avl_write_req,
    output [6:0] avl_size);

    logic [13:2] program_rom_interface_addr;
    logic program_rom_read_req;
    logic [31:0] program_rom_read_data;
    logic program_rom_read_data_valid;
    program_rom_interface program_rom_interface0(
        .reset_n(reset_n),
        .clk(clk),

        .addr(program_rom_interface_addr),
        .read_req(program_rom_read_req),
        .read_data(program_rom_read_data),
        .read_data_valid(program_rom_read_data_valid),
        
        .program_rom_addr(program_rom_addr),
        .program_rom_q(program_rom_q));

    logic [31:0] led_interface_write_data;
    logic [3:0] led_interface_byte_enable;
    logic led_interface_write_req;
    logic led_interface_read_req;
    logic [31:0] led_interface_read_data;
    logic led_interface_read_data_valid;
    led_interface led_interface0(
        .reset_n(reset_n),
        .clk(clk),

        .write_data(led_interface_write_data),
        .byte_enable(led_interface_byte_enable),
        .write_req(led_interface_write_req),
        .read_req(led_interface_read_req),
        .read_data(led_interface_read_data),
        .read_data_valid(led_interface_read_data_valid),

        .leds(leds));

    logic uart_transmitter_ready;
    uart_transmitter uart_transmitter0(
        .reset_n(reset_n),
        .clk(clk),

        .write_data(uart_transmitter_interface_uart_write_data),
        .write_req(uart_transmitter_interface_uart_write_req),
        .ready(uart_transmitter_ready),

        .tx(uart_tx));

    logic uart_transmitter_interface_addr;
    logic [31:0] uart_transmitter_interface_write_data;
    logic [3:0] uart_transmitter_interface_byte_enable;
    logic uart_transmitter_interface_write_req;
    logic uart_transmitter_interface_read_req;
    logic [31:0] uart_transmitter_interface_read_data;
    logic uart_transmitter_interface_read_data_valid;
    logic [7:0] uart_transmitter_interface_uart_write_data;
    logic uart_transmitter_interface_uart_write_req;
    uart_transmitter_interface uart_transmitter_interface0(
        .reset_n(reset_n),
        .clk(clk),

        .addr(uart_transmitter_interface_addr),
        .write_data(uart_transmitter_interface_write_data),
        .byte_enable(uart_transmitter_interface_byte_enable),
        .write_req(uart_transmitter_interface_write_req),
        .read_req(uart_transmitter_interface_read_req),
        .read_data(uart_transmitter_interface_read_data),
        .read_data_valid(uart_transmitter_interface_read_data_valid),

        .uart_write_data(uart_transmitter_interface_uart_write_data),
        .uart_write_req(uart_transmitter_interface_uart_write_req),
        .uart_ready(uart_transmitter_ready));

    logic ddr3_interface_ready;
    logic [26:2] ddr3_interface_addr;
    logic [31:0] ddr3_interface_write_data;
    logic [3:0] ddr3_interface_byte_enable;
    logic ddr3_interface_write_req;
    logic ddr3_interface_read_req;
    logic [31:0] ddr3_interface_read_data;
    logic ddr3_interface_read_data_valid;
    ddr3_interface ddr3_interface0(
        .reset_n(reset_n),
        .clk(clk),

        .ready(ddr3_interface_ready),
        .addr(ddr3_interface_addr),
        .write_data(ddr3_interface_write_data),
        .byte_enable(ddr3_interface_byte_enable),
        .write_req(ddr3_interface_write_req),
        .read_req(ddr3_interface_read_req),
        .read_data(ddr3_interface_read_data),
        .read_data_valid(ddr3_interface_read_data_valid),

        .avl_ready(avl_ready),
        .avl_burstbegin(avl_burstbegin),
        .avl_addr(avl_addr),
        .avl_rdata_valid(avl_rdata_valid),
        .avl_rdata(avl_rdata),
        .avl_wdata(avl_wdata),
        .avl_be(avl_be),
        .avl_read_req(avl_read_req),
        .avl_write_req(avl_write_req),
        .avl_size(avl_size));

    logic system_bus_ready;
    logic [31:2] system_bus_addr;
    logic [31:0] system_bus_write_data;
    logic [3:0] system_bus_byte_enable;
    logic system_bus_write_req;
    logic system_bus_read_req;
    logic [31:0] system_bus_read_data;
    logic system_bus_read_data_valid;
    system_bus system_bus0(
        .reset_n(reset_n),
        .clk(clk),

        .ready(system_bus_ready),
        .addr(system_bus_addr),
        .write_data(system_bus_write_data),
        .byte_enable(system_bus_byte_enable),
        .write_req(system_bus_write_req),
        .read_req(system_bus_read_req),
        .read_data(system_bus_read_data),
        .read_data_valid(system_bus_read_data_valid),

        .program_rom_interface_addr(program_rom_interface_addr),
        .program_rom_interface_read_req(program_rom_read_req),
        .program_rom_interface_read_data(program_rom_read_data),
        .program_rom_interface_read_data_valid(program_rom_read_data_valid),

        .led_interface_write_data(led_interface_write_data),
        .led_interface_byte_enable(led_interface_byte_enable),
        .led_interface_write_req(led_interface_write_req),
        .led_interface_read_req(led_interface_read_req),
        .led_interface_read_data(led_interface_read_data),
        .led_interface_read_data_valid(led_interface_read_data_valid),

        .uart_transmitter_interface_addr(uart_transmitter_interface_addr),
        .uart_transmitter_interface_write_data(uart_transmitter_interface_write_data),
        .uart_transmitter_interface_byte_enable(uart_transmitter_interface_byte_enable),
        .uart_transmitter_interface_write_req(uart_transmitter_interface_write_req),
        .uart_transmitter_interface_read_req(uart_transmitter_interface_read_req),
        .uart_transmitter_interface_read_data(uart_transmitter_interface_read_data),
        .uart_transmitter_interface_read_data_valid(uart_transmitter_interface_read_data_valid),

        .ddr3_interface_ready(ddr3_interface_ready),
        .ddr3_interface_addr(ddr3_interface_addr),
        .ddr3_interface_write_data(ddr3_interface_write_data),
        .ddr3_interface_byte_enable(ddr3_interface_byte_enable),
        .ddr3_interface_write_req(ddr3_interface_write_req),
        .ddr3_interface_read_req(ddr3_interface_read_req),
        .ddr3_interface_read_data(ddr3_interface_read_data),
        .ddr3_interface_read_data_valid(ddr3_interface_read_data_valid));

    cpu cpu0(
        .reset_n(reset_n),
        .clk(clk),

        .system_bus_ready(system_bus_ready),
        .system_bus_addr(system_bus_addr),
        .system_bus_write_data(system_bus_write_data),
        .system_bus_byte_enable(system_bus_byte_enable),
        .system_bus_write_req(system_bus_write_req),
        .system_bus_read_req(system_bus_read_req),
        .system_bus_read_data(system_bus_read_data),
        .system_bus_read_data_valid(system_bus_read_data_valid));

endmodule
