`default_nettype none

module control(
    input reset_n,
    input clk,

    input instruction_fetch_ready,
    output instruction_fetch_enable,

    input decode_ready);

    localparam STATE_INSTRUCTION_FETCH = 3'h0;
    localparam STATE_DECODE = 3'h1;
    localparam STATE_EXECUTE = 3'h2;
    localparam STATE_MEM_ISSUE = 3'h3;
    localparam STATE_MEM_WAIT = 3'h4;
    localparam STATE_WRITEBACK = 3'h5;
    localparam STATE_ERROR = 3'h6;
    logic [2:0] state;
    logic [2:0] state_next;

    assign instruction_fetch_enable = state == STATE_INSTRUCTION_FETCH;

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
                    state_next = STATE_EXECUTE;
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
