`default_nettype none

module reset_synchronizer(
    input wire logic reset_n,
    input wire logic clk,

    output wire logic reset_n_sync);

    parameter STAGES = 2;

    (* ASYNC_REG = "TRUE" *) logic [STAGES - 1:0] sync_chain;

    always_ff @(posedge clk, negedge reset_n) begin
        if (~reset_n) begin
            sync_chain <= {STAGES{1'd0}};
        end
        else begin
            sync_chain <= {sync_chain[STAGES - 2:0], 1'd1};
        end
    end

    assign reset_n_sync = sync_chain[STAGES - 1];

endmodule
