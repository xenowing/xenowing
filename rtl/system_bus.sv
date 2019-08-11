`default_nettype none

module system_bus(
    input reset_n,
    input clk,

    output ready,
    input [31:2] addr,
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
    input led_interface_read_data_valid,

    output uart_transmitter_interface_addr,
    output [31:0] uart_transmitter_interface_write_data,
    output [3:0] uart_transmitter_interface_byte_enable,
    output uart_transmitter_interface_write_req,
    output uart_transmitter_interface_read_req,
    input [31:0] uart_transmitter_interface_read_data,
    input uart_transmitter_interface_read_data_valid,

    input ddr3_interface_ready,
    output [26:2] ddr3_interface_addr,
    output [31:0] ddr3_interface_write_data,
    output [3:0] ddr3_interface_byte_enable,
    output ddr3_interface_write_req,
    output ddr3_interface_read_req,
    input [31:0] ddr3_interface_read_data,
    input ddr3_interface_read_data_valid);

    logic dummy_read_data_valid;
    logic dummy_read_data_valid_next;

    assign program_rom_interface_addr = addr[13:2];

    assign led_interface_write_data = write_data;
    assign led_interface_byte_enable = byte_enable;

    assign uart_transmitter_interface_addr = addr[2];
    assign uart_transmitter_interface_write_data = write_data;
    assign uart_transmitter_interface_byte_enable = byte_enable;

    assign ddr3_interface_addr = addr[26:2];
    assign ddr3_interface_write_data = write_data;
    assign ddr3_interface_byte_enable = byte_enable;

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

        if (uart_transmitter_interface_read_data_valid) begin
            read_data = uart_transmitter_interface_read_data;
            read_data_valid = 1;
        end

        if (ddr3_interface_read_data_valid) begin
            read_data = ddr3_interface_read_data;
            read_data_valid = 1;
        end

        program_rom_interface_read_req = 0;

        led_interface_write_req = 0;
        led_interface_read_req = 0;

        uart_transmitter_interface_write_req = 0;
        uart_transmitter_interface_read_req = 0;

        ddr3_interface_write_req = 0;
        ddr3_interface_read_req = 0;

        case (addr[31:28])
            4'h1: program_rom_interface_read_req = read_req;

            4'h2: begin
                if (!addr[24]) begin
                    led_interface_write_req = write_req;
                    led_interface_read_req = read_req;
                end
                else begin
                    uart_transmitter_interface_write_req = write_req;
                    uart_transmitter_interface_read_req = read_req;
                end
            end

            4'h3: begin
                ready = ddr3_interface_ready;
                ddr3_interface_write_req = write_req;
                ddr3_interface_read_req = read_req;
            end

            default: begin
                if (read_req) begin
                    dummy_read_data_valid_next = 1;
                end
            end
        endcase
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            dummy_read_data_valid <= 0;
        end
        else begin
            dummy_read_data_valid <= dummy_read_data_valid_next;
        end
    end

endmodule
