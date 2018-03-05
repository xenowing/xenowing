`default_nettype none

module rom_loader(
    input reset_n,
    input clk,

    input [7:0] uart_receiver_data,
    input uart_receiver_data_ready,

    output system_soft_reset,

    output logic [13:0] program_rom_write_addr,
    output logic [31:0] program_rom_write_data,
    output logic program_rom_write_req);

    logic [13:0] program_rom_write_addr_next;
    logic [31:0] program_rom_write_data_next;
    logic program_rom_write_req_next;

    localparam STATE_RECEIVE_LEN = 1'h0;
    localparam STATE_RECEIVE_DATA = 1'h1;
    logic state;
    logic state_next;

    logic [31:0] len;
    logic [31:0] len_next;
    logic [1:0] len_receive_index;
    logic [1:0] len_receive_index_next;

    logic [31:0] data_receive_index;
    logic [31:0] data_receive_index_next;

    assign system_soft_reset = state == STATE_RECEIVE_DATA;

    always_comb begin
        program_rom_write_addr_next = program_rom_write_addr;
        program_rom_write_data_next = program_rom_write_data;
        program_rom_write_req_next = program_rom_write_req;

        state_next = state;

        len_next = len;
        len_receive_index_next = len_receive_index;

        data_receive_index_next = data_receive_index;

        program_rom_write_req_next = 0;

        case (state)
            STATE_RECEIVE_LEN: begin
                if (uart_receiver_data_ready) begin
                    len_next = {uart_receiver_data, len_next[31:8]};

                    if (len_receive_index == 2'h3) begin
                        state_next = STATE_RECEIVE_DATA;
                        data_receive_index_next = 32'h0;
                    end
                    else begin
                        len_receive_index_next = len_receive_index + 2'h1;
                    end
                end
            end

            STATE_RECEIVE_DATA: begin
                if (uart_receiver_data_ready) begin
                    program_rom_write_data_next = {uart_receiver_data, program_rom_write_data[31:8]};

                    if (data_receive_index[1:0] == 2'h3) begin
                        program_rom_write_addr_next = data_receive_index[15:2];
                        program_rom_write_req_next = 1;
                    end

                    data_receive_index_next = data_receive_index + 32'h1;
                    if (data_receive_index_next == len) begin
                        state_next = STATE_RECEIVE_LEN;
                        len_receive_index_next = 2'h0;
                    end
                end
            end
        endcase
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            program_rom_write_addr <= 14'h0;
            program_rom_write_data <= 32'h0;
            program_rom_write_req <= 0;

            state <= STATE_RECEIVE_LEN;

            len <= 32'h0;
            len_receive_index <= 2'h0;

            data_receive_index <= 32'h0;
        end
        else begin
            program_rom_write_addr <= program_rom_write_addr_next;
            program_rom_write_data <= program_rom_write_data_next;
            program_rom_write_req <= program_rom_write_req_next;

            state <= state_next;

            len <= len_next;
            len_receive_index <= len_receive_index_next;

            data_receive_index <= data_receive_index_next;
        end
    end

endmodule
