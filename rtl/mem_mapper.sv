`default_nettype none

module mem_mapper(
    input reset_n,
    input clk,

    output ready,
    input [31:0] addr,
    input [31:0] write_data,
    input [3:0] byte_enable,
    input write_req,
    input read_req,
    output [31:0] read_data,
    output read_data_valid,

    output [13:0] program_rom_interface_addr,
    output program_rom_interface_read_req,
    input [31:0] program_rom_interface_read_data,
    input program_rom_interface_read_data_valid,

    output [31:0] led_interface_write_data,
    output [3:0] led_interface_byte_enable,
    output led_interface_write_req,
    output led_interface_read_req,
    input [31:0] led_interface_read_data,
    input led_interface_read_data_valid);

    logic dummy_read_data_valid;
    logic dummy_read_data_valid_next;

    assign program_rom_interface_addr = addr[13:0];

    assign led_interface_byte_enable = byte_enable;

    always_comb begin
        dummy_read_data_valid_next = dummy_read_data_valid;

        dummy_read_data_valid_next = 0;

        ready = 1;
        read_data = 32'h0;
        read_data_valid = 0;

        if (dummy_read_data_valid) begin
            read_data_valid = 1;
        end

        if (program_rom_interface_read_data_valid) begin
            read_data = program_rom_interface_read_data;
            read_data_valid = 1;
        end

        if (led_interface_read_data_valid) begin
            read_data = led_interface_read_data;
            read_data_valid = 1;
        end

        program_rom_interface_read_req = 0;

        led_interface_write_data = 32'h0;
        led_interface_write_req = 0;
        led_interface_read_req = 0;

        case (addr[31:28])
            4'h1: program_rom_interface_read_req = read_req;

            4'h2: begin
                led_interface_write_data = write_data;
                led_interface_write_req = write_req;
                led_interface_read_req = read_req;
            end

            default: begin
                if (read_req) begin
                    dummy_read_data_valid_next = 1;
                end
            end
        endcase
    end

    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            dummy_read_data_valid <= 0;
        end
        else begin
            dummy_read_data_valid <= dummy_read_data_valid_next;
        end
    end

endmodule
