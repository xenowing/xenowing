`default_nettype none

module store_unit(
    input clk,
    input reset_n,

    output write_ready,
    input write_req,
    input [31:0] write_addr,
    input [31:0] write_data,
    input [3:0] write_byte_enable,

    input mem_ready,
    output logic [31:0] mem_addr,
    output logic [31:0] mem_write_data,
    output logic [3:0] mem_byte_enable,
    output logic mem_write_req);

    logic [31:0] mem_addr_next;
    logic [31:0] mem_write_data_next;
    logic [3:0] mem_byte_enable_next;
    logic mem_write_req_next;

    logic [63:0] write_word;
    logic [63:0] write_word_next;
    logic [7:0] write_word_byte_enable;
    logic [7:0] write_word_byte_enable_next;

    localparam STATE_IDLE = 2'h0;
    localparam STATE_STORE_LOW = 2'h1;
    localparam STATE_STORE_HIGH = 2'h2;
    logic [1:0] state;
    logic [1:0] state_next;

    assign write_ready = state == STATE_IDLE;

    always_comb begin
        mem_addr_next = mem_addr;
        mem_write_data_next = mem_write_data;
        mem_byte_enable_next = mem_byte_enable;
        mem_write_req_next = mem_write_req;

        write_word_next = write_word;
        write_word_byte_enable_next = write_word_byte_enable;

        state_next = state;

        case (state)
            STATE_IDLE: begin
                if (write_req) begin
                    write_word_next = {32'h0, write_data} << {write_addr[1:0], 3'h0};
                    write_word_byte_enable_next = {4'h0, write_byte_enable} << write_addr[1:0];

                    mem_addr_next = {write_addr[31:2], 2'h0};
                    mem_write_data_next = write_word_next[31:0];
                    mem_byte_enable_next = write_word_byte_enable_next[3:0];
                    mem_write_req_next = 1;

                    state_next = STATE_STORE_LOW;
                end
            end

            STATE_STORE_LOW: begin
                if (mem_ready) begin
                    if (write_word_byte_enable[7:4] == 4'h0) begin
                        mem_write_req_next = 0;

                        state_next = STATE_IDLE;
                    end
                    else begin
                        mem_addr_next = {mem_addr[31:2] + 30'h1, 2'h0};
                        mem_write_data_next = write_word_next[63:32];
                        mem_byte_enable_next = write_word_byte_enable[7:4];

                        state_next = STATE_STORE_HIGH;
                    end
                end
            end

            STATE_STORE_HIGH: begin
                if (mem_ready) begin
                    mem_write_req_next = 0;

                    state_next = STATE_IDLE;
                end
            end
        endcase
    end

    always @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            mem_addr <= 32'h0;
            mem_write_data <= 32'h0;
            mem_byte_enable <= 4'h0;
            mem_write_req <= 0;

            write_word <= 64'h0;
            write_word_byte_enable <= 8'h0;

            state <= STATE_IDLE;
        end
        else begin
            mem_addr <= mem_addr_next;
            mem_write_data <= mem_write_data_next;
            mem_byte_enable <= mem_byte_enable_next;
            mem_write_req <= mem_write_req_next;

            write_word <= write_word_next;
            write_word_byte_enable <= write_word_byte_enable_next;

            state <= state_next;
        end
    end

endmodule
