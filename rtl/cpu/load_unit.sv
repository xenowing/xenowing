`default_nettype none

module load_unit(
    input reset_n,
    input clk,

    output read_ready,
    input read_req,
    input [31:0] read_addr,
    input [3:0] read_byte_enable,
    output logic [31:0] read_data,
    output logic read_data_valid,

    input mem_ready,
    output logic [31:0] mem_addr,
    output logic [3:0] mem_byte_enable,
    output logic mem_read_req,
    input [31:0] mem_read_data,
    input mem_read_data_valid);

    logic [31:0] read_data_next;
    logic read_data_valid_next;

    logic [31:0] mem_addr_next;
    logic [3:0] mem_byte_enable_next;
    logic mem_read_req_next;

    logic [7:0] read_word_byte_enable;
    logic [7:0] read_word_byte_enable_next;

    localparam STATE_IDLE = 2'h0;
    localparam STATE_LOAD_LOW = 2'h1;
    localparam STATE_LOAD_HIGH = 2'h2;
    logic [1:0] state;
    logic [1:0] state_next;

    logic [1:0] read_addr_low;
    logic [1:0] read_addr_low_next;

    assign read_ready = state == STATE_IDLE;

    logic read_buffer_clear;
    logic read_buffer_clear_next;

    logic [31:0] read_buffer_data[0:1];
    logic [1:0] read_buffer_count;

    read_buffer read_buffer0(
        .clk(clk),
        .reset_n(reset_n),

        .clear(read_buffer_clear),

        .mem_read_data(mem_read_data),
        .mem_read_data_valid(mem_read_data_valid),

        .data(read_buffer_data),
        .count(read_buffer_count));

    always_comb begin
        read_data_next = read_data;
        read_data_valid_next = read_data_valid;

        mem_addr_next = mem_addr;
        mem_byte_enable_next = mem_byte_enable;
        mem_read_req_next = mem_read_req;

        read_word_byte_enable_next = read_word_byte_enable;

        state_next = state;

        read_addr_low_next = read_addr_low;

        read_buffer_clear_next = read_buffer_clear;

        read_data_valid_next = 0;

        read_buffer_clear_next = 0;

        case (state)
            STATE_IDLE: begin
                if (read_req) begin
                    read_addr_low_next = read_addr[1:0];

                    read_word_byte_enable_next = {4'h0, read_byte_enable} << read_addr_low_next;

                    mem_addr_next = {read_addr[31:2], 2'h0};
                    mem_byte_enable_next = read_word_byte_enable_next[3:0];
                    mem_read_req_next = 1;

                    read_buffer_clear_next = 1;

                    state_next = STATE_LOAD_LOW;
                end
            end

            STATE_LOAD_LOW: begin
                if (mem_ready) begin
                    if (read_word_byte_enable[7:4] == 4'h0) begin
                        mem_read_req_next = 0;

                        if (read_buffer_count == 2'h1) begin
                            read_data_next = {read_buffer_data[1], read_buffer_data[0]} >> {read_addr_low, 3'h0};
                            read_data_valid_next = 1;

                            state_next = STATE_IDLE;
                        end
                    end
                    else begin
                        mem_addr_next = {mem_addr[31:2] + 30'h1, 2'h0};
                        mem_byte_enable_next = read_word_byte_enable[7:4];

                        state_next = STATE_LOAD_HIGH;
                    end
                end
            end

            STATE_LOAD_HIGH: begin
                if (mem_ready) begin
                    mem_read_req_next = 0;
                end

                if (!mem_read_req_next) begin
                    if (read_buffer_count == 2'h2) begin
                        read_data_next = {read_buffer_data[1], read_buffer_data[0]} >> {read_addr_low, 3'h0};
                        read_data_valid_next = 1;

                        state_next = STATE_IDLE;
                    end
                end
            end
        endcase
    end

    always @(posedge clk) begin
        if (!reset_n) begin
            read_data <= 32'h0;
            read_data_valid <= 0;

            mem_addr <= 32'h0;
            mem_byte_enable <= 4'h0;
            mem_read_req <= 0;

            read_word_byte_enable <= 8'h0;

            state <= STATE_IDLE;

            read_addr_low <= 2'h0;

            read_buffer_clear <= 0;
        end
        else begin
            read_data <= read_data_next;
            read_data_valid <= read_data_valid_next;

            mem_addr <= mem_addr_next;
            mem_byte_enable <= mem_byte_enable_next;
            mem_read_req <= mem_read_req_next;

            read_word_byte_enable <= read_word_byte_enable_next;

            state <= state_next;

            read_addr_low <= read_addr_low_next;

            read_buffer_clear <= read_buffer_clear_next;
        end
    end

endmodule
