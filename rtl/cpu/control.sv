`default_nettype none

module control(
    input reset_n,
    input clk,

    input instruction_fetch_ready,
    output instruction_fetch_enable,

    input decode_ready,
    output decode_enable,

    input execute_mem_ready,
    output execute_mem_enable,

    input writeback_ready,
    output writeback_enable);

    localparam STATE_INSTRUCTION_FETCH = 3'h0;
    localparam STATE_DECODE = 3'h1;
    localparam STATE_EXECUTE_MEM = 3'h2;
    localparam STATE_WRITEBACK = 3'h3;
    localparam STATE_ERROR = 3'h4;
    logic [2:0] state;
    logic [2:0] state_next;

    assign instruction_fetch_enable = state == STATE_INSTRUCTION_FETCH;
    assign decode_enable = state == STATE_DECODE;
    assign execute_mem_enable = state == STATE_EXECUTE_MEM;
    assign writeback_enable = state == STATE_WRITEBACK;

    always_comb begin
        state_next = state;

        case (state)
            STATE_INSTRUCTION_FETCH: begin
                if (instruction_fetch_ready) begin
                    state_next = STATE_DECODE;
                end
            end

            STATE_DECODE: begin
                if (decode_ready) begin
                    state_next = STATE_EXECUTE_MEM;
                end
            end

            STATE_EXECUTE_MEM: begin
                if (execute_mem_ready) begin
                    state_next = STATE_WRITEBACK;
                end
            end

            STATE_WRITEBACK: begin
                if (writeback_ready) begin
                    state_next = STATE_INSTRUCTION_FETCH;
                end
            end

            STATE_ERROR: begin
                // TODO
            end
        endcase
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            state <= STATE_INSTRUCTION_FETCH;
        end
        else begin
            state <= state_next;
        end
    end

endmodule
