`default_nettype none

module uart_transmitter_interface(
    input reset_n,
    input clk,

    input addr,
    input [31:0] write_data,
    input [3:0] byte_enable,
    input write_req,
    input read_req,
    output [31:0] read_data,
    output logic read_data_valid,

    output [7:0] uart_write_data,
    output logic uart_write_req,
    input uart_ready);

    assign read_data = {31'h0, uart_ready};

    logic read_data_valid_next;

    assign uart_write_data = write_data[7:0];

    logic uart_write_req_next;

    always_comb begin
        read_data_valid_next = read_data_valid;

        uart_write_req_next = uart_write_req;

        read_data_valid_next = read_req;

        uart_write_req_next = 0;

        if (write_req) begin
            if (addr && byte_enable[0]) begin
                // Write data reg
                if (uart_ready) begin
                    uart_write_req_next = 1;
                end
            end
        end
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            read_data_valid <= 0;

            uart_write_req <= 0;
        end
        else begin
            read_data_valid <= read_data_valid_next;

            uart_write_req <= uart_write_req_next;
        end
    end

endmodule
