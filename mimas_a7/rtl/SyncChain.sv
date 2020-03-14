`default_nettype none

module SyncChain#(parameter STAGES = 2, DEFAULT = 1'b0) (
    input wire logic reset_n,
    input wire logic clk,

    input wire logic x,

    output wire logic x_sync);

    (* ASYNC_REG = "TRUE" *) logic [STAGES - 1:0] sync_chain;

    always_ff @(posedge clk, negedge reset_n) begin
        if (~reset_n) begin
            sync_chain <= {STAGES{DEFAULT}};
        end
        else begin
            sync_chain <= {sync_chain[STAGES - 2:0], x};
        end
    end

    assign x_sync = sync_chain[STAGES - 1];

endmodule
