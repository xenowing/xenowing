`default_nettype none

module reset_synchronizer(
    input wire reset_n,
    input wire clk,

    output wire reset_n_sync);

    logic sync_chain1, sync_chain2;
    always_ff @(posedge clk, negedge reset_n) begin
        if (~reset_n) begin
            { sync_chain1, sync_chain2 } <= 2'd0;
        end
        else begin
            { sync_chain2, sync_chain1 } <= { sync_chain1, 1'd1 };
        end
    end

    assign reset_n_sync = sync_chain2;

endmodule
