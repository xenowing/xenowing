`default_nettype none

module xenowing(
    input reset_n,
    input clk,

    output [13:0] program_rom_addr,
    input [31:0] program_rom_q,

    output [2:0] leds);

    logic [13:0] program_rom_interface_addr;
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

    logic mem_mapper_ready;
    logic [31:0] mem_mapper_addr;
    logic [31:0] mem_mapper_write_data;
    logic [3:0] mem_mapper_byte_enable;
    logic mem_mapper_write_req;
    logic mem_mapper_read_req;
    logic [31:0] mem_mapper_read_data;
    logic mem_mapper_read_data_valid;
    mem_mapper mem_mapper0(
        .reset_n(reset_n),
        .clk(clk),

        .ready(mem_mapper_ready),
        .addr(mem_mapper_addr),
        .write_data(mem_mapper_write_data),
        .byte_enable(mem_mapper_byte_enable),
        .write_req(mem_mapper_write_req),
        .read_req(mem_mapper_read_req),
        .read_data(mem_mapper_read_data),
        .read_data_valid(mem_mapper_read_data_valid),

        .program_rom_interface_addr(program_rom_interface_addr),
        .program_rom_interface_read_req(program_rom_read_req),
        .program_rom_interface_read_data(program_rom_read_data),
        .program_rom_interface_read_data_valid(program_rom_read_data_valid),

        .led_interface_write_data(led_interface_write_data),
        .led_interface_byte_enable(led_interface_byte_enable),
        .led_interface_write_req(led_interface_write_req),
        .led_interface_read_req(led_interface_read_req),
        .led_interface_read_data(led_interface_read_data),
        .led_interface_read_data_valid(led_interface_read_data_valid));

    cpu cpu0(
        .reset_n(reset_n),
        .clk(clk),

        .ready(mem_mapper_ready),
        .addr(mem_mapper_addr),
        .write_data(mem_mapper_write_data),
        .byte_enable(mem_mapper_byte_enable),
        .write_req(mem_mapper_write_req),
        .read_req(mem_mapper_read_req),
        .read_data(mem_mapper_read_data),
        .read_data_valid(mem_mapper_read_data_valid));

endmodule
