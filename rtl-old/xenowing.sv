`default_nettype none

module xenowing(
    input reset_n,
    input clk,

    output [10:0] program_rom_addr,
    input [31:0] program_rom_q,

    output [13:0] ram_address,
    output [7:0] ram_byteena,
    output [63:0] ram_data,
    output ram_wren,
    input [63:0] ram_q,

    output [2:0] leds,

    output uart_tx,

    output display_pixel_clk,
    output display_vsync,
    output display_hsync,
    output display_data_enable,
    output [23:0] display_pixel_data);

    logic [10:0] program_rom_interface_addr;
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

    logic [2:0] led_interface_write_data;
    logic led_interface_byte_enable;
    logic led_interface_write_req;
    logic led_interface_read_req;
    logic [2:0] led_interface_read_data;
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
    logic [7:0] uart_transmitter_interface_write_data;
    logic uart_transmitter_interface_byte_enable;
    logic uart_transmitter_interface_write_req;
    logic uart_transmitter_interface_read_req;
    logic uart_transmitter_interface_read_data;
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

    logic [5:0] display_buffer_write_addr;
    logic [63:0] display_buffer_write_data;
    logic display_buffer_write_enable;
    logic [5:0] display_buffer_read_addr;
    logic [63:0] display_buffer_read_data;
    display_buffer display_buffer0(
        .clk(clk),

        .write_addr(display_buffer_write_addr),
        .write_data(display_buffer_write_data),
        .write_enable(display_buffer_write_enable),
        .read_addr(display_buffer_read_addr),
        .read_data(display_buffer_read_data));

    logic [13:0] display_load_issue_framebuffer_base_addr_data;
    logic display_load_issue_framebuffer_base_addr_write_enable;
    logic [13:0] display_load_issue_bus_read_addr;
    logic [7:0] display_load_issue_bus_byte_enable;
    logic display_load_issue_start;
    logic display_load_issue_bus_ready;
    logic display_load_issue_bus_read_addr_reset;
    logic display_load_issue_bus_read_req;
    display_load_issue display_load_issue0(
        .reset_n(reset_n),
        .clk(clk),

        .framebuffer_base_addr_data(display_load_issue_framebuffer_base_addr_data),
        .framebuffer_base_addr_write_enable(display_load_issue_framebuffer_base_addr_write_enable),

        .bus_read_addr(display_load_issue_bus_read_addr),
        .bus_byte_enable(display_load_issue_bus_byte_enable),
        .start(display_load_issue_start),
        .bus_ready(display_load_issue_bus_ready),
        .bus_read_addr_reset(display_load_issue_bus_read_addr_reset),
        .bus_read_req(display_load_issue_bus_read_req));

    logic display_load_return_start;
    logic [63:0] display_load_return_bus_read_data;
    logic display_load_return_bus_read_data_valid;
    logic [5:0] display_load_return_load_addr;
    logic [63:0] display_load_return_load_data;
    logic display_load_return_load_data_valid;
    display_load_return display_load_return0(
        .reset_n(reset_n),
        .clk(clk),

        .start(display_load_return_start),

        .bus_read_data(display_load_return_bus_read_data),
        .bus_read_data_valid(display_load_return_bus_read_data_valid),

        .load_addr(display_load_return_load_addr),
        .load_data(display_load_return_load_data),
        .load_data_valid(display_load_return_load_data_valid));

    logic display_load_start;
    logic display_vblank;
    display display0(
        .reset_n(reset_n),
        .clk(clk),

        .buffer_addr(display_buffer_read_addr),
        .buffer_data(display_buffer_read_data),

        .load_bus_read_addr_reset(display_load_issue_bus_read_addr_reset),
        .load_start(display_load_start),

        .vblank(display_vblank),

        .pixel_clk(display_pixel_clk),
        .vsync(display_vsync),
        .hsync(display_hsync),
        .data_enable(display_data_enable),
        .pixel_data(display_pixel_data));

    assign display_buffer_write_addr = display_load_return_load_addr;
    assign display_buffer_write_data = display_load_return_load_data;
    assign display_buffer_write_enable = display_load_return_load_data_valid;

    assign display_load_issue_start = display_load_start;

    assign display_load_return_start = display_load_start;

    logic display_interface_addr;
    logic [16:0] display_interface_write_data;
    logic display_interface_byte_enable;
    logic display_interface_write_req;
    logic display_interface_read_req;
    logic [1:0] display_interface_read_data;
    logic display_interface_read_data_valid;
    display_interface display_interface0(
        .reset_n(reset_n),
        .clk(clk),

        .addr(display_interface_addr),
        .write_data(display_interface_write_data),
        .byte_enable(display_interface_byte_enable),
        .write_req(display_interface_write_req),
        .read_req(display_interface_read_req),
        .read_data(display_interface_read_data),
        .read_data_valid(display_interface_read_data_valid),

        .display_vblank(display_vblank),

        .display_load_issue_framebuffer_base_addr_data(display_load_issue_framebuffer_base_addr_data),
        .display_load_issue_framebuffer_base_addr_write_enable(display_load_issue_framebuffer_base_addr_write_enable));

    logic cpu_ram_interface_ready;
    logic [14:0] cpu_ram_interface_addr;
    logic [31:0] cpu_ram_interface_write_data;
    logic [3:0] cpu_ram_interface_byte_enable;
    logic cpu_ram_interface_write_req;
    logic cpu_ram_interface_read_req;
    logic [31:0] cpu_ram_interface_read_data;
    logic cpu_ram_interface_read_data_valid;
    logic cpu_ram_interface_avl_ready;
    logic [13:0] cpu_ram_interface_avl_addr;
    logic cpu_ram_interface_avl_rdata_valid;
    logic [63:0] cpu_ram_interface_avl_rdata;
    logic [63:0] cpu_ram_interface_avl_wdata;
    logic [7:0] cpu_ram_interface_avl_be;
    logic cpu_ram_interface_avl_read_req;
    logic cpu_ram_interface_avl_write_req;
    cpu_ram_interface cpu_ram_interface0(
        .reset_n(reset_n),
        .clk(clk),

        .ready(cpu_ram_interface_ready),
        .addr(cpu_ram_interface_addr),
        .write_data(cpu_ram_interface_write_data),
        .byte_enable(cpu_ram_interface_byte_enable),
        .write_req(cpu_ram_interface_write_req),
        .read_req(cpu_ram_interface_read_req),
        .read_data(cpu_ram_interface_read_data),
        .read_data_valid(cpu_ram_interface_read_data_valid),

        .avl_ready(cpu_ram_interface_avl_ready),
        .avl_addr(cpu_ram_interface_avl_addr),
        .avl_rdata_valid(cpu_ram_interface_avl_rdata_valid),
        .avl_rdata(cpu_ram_interface_avl_rdata),
        .avl_wdata(cpu_ram_interface_avl_wdata),
        .avl_be(cpu_ram_interface_avl_be),
        .avl_read_req(cpu_ram_interface_avl_read_req),
        .avl_write_req(cpu_ram_interface_avl_write_req));

    ram_interface ram_interface0(
        .reset_n(reset_n),
        .clk(clk),

        .cpu_avl_ready(cpu_ram_interface_avl_ready),
        .cpu_avl_addr(cpu_ram_interface_avl_addr),
        .cpu_avl_rdata_valid(cpu_ram_interface_avl_rdata_valid),
        .cpu_avl_rdata(cpu_ram_interface_avl_rdata),
        .cpu_avl_wdata(cpu_ram_interface_avl_wdata),
        .cpu_avl_be(cpu_ram_interface_avl_be),
        .cpu_avl_read_req(cpu_ram_interface_avl_read_req),
        .cpu_avl_write_req(cpu_ram_interface_avl_write_req),

        .display_avl_ready(display_load_issue_bus_ready),
        .display_avl_addr(display_load_issue_bus_read_addr),
        .display_avl_rdata_valid(display_load_return_bus_read_data_valid),
        .display_avl_rdata(display_load_return_bus_read_data),
        .display_avl_be(display_load_issue_bus_byte_enable),
        .display_avl_read_req(display_load_issue_bus_read_req),

        .ram_address(ram_address),
        .ram_byteena(ram_byteena),
        .ram_data(ram_data),
        .ram_wren(ram_wren),
        .ram_q(ram_q));

    logic system_bus_ready;
    logic [29:0] system_bus_addr;
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

        .display_interface_addr(display_interface_addr),
        .display_interface_write_data(display_interface_write_data),
        .display_interface_byte_enable(display_interface_byte_enable),
        .display_interface_write_req(display_interface_write_req),
        .display_interface_read_req(display_interface_read_req),
        .display_interface_read_data(display_interface_read_data),
        .display_interface_read_data_valid(display_interface_read_data_valid),

        .ram_interface_ready(cpu_ram_interface_ready),
        .ram_interface_addr(cpu_ram_interface_addr),
        .ram_interface_write_data(cpu_ram_interface_write_data),
        .ram_interface_byte_enable(cpu_ram_interface_byte_enable),
        .ram_interface_write_req(cpu_ram_interface_write_req),
        .ram_interface_read_req(cpu_ram_interface_read_req),
        .ram_interface_read_data(cpu_ram_interface_read_data),
        .ram_interface_read_data_valid(cpu_ram_interface_read_data_valid));

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
