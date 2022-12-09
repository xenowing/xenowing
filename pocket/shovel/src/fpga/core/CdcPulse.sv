`default_nettype none

// Detects a single-cycle pulse in the source domain and safely encodes, synchronizes, and
//  decodes it into a matching single-cycle pulse in the target clock domain.
// Assumes pulses will be generated relatively seldomly with respect to the destination clock.
module CdcPulse(
    input wire logic reset_n,

    input wire logic src_clk,
    input wire logic src_pulse,

    input wire logic dst_clk,
    output logic dst_pulse
);

// Convert source pulses to level edges
logic src_level;
always_ff @(posedge src_clk, negedge reset_n) begin
    if (~reset_n) begin
        src_level <= 1'b0;
    end
    else begin
        src_level <= src_level ^ src_pulse;
    end
end

// Synchronize level to target domain
logic dst_level;
SyncChain #(
    .STAGES(3)
) sync_chain (
    .reset_n(reset_n),
    .clk(dst_clk),

    .x(src_level),

    .x_sync(dst_level)
);

// Decode level edges to destination pulses
logic prev_dst_level;
always_ff @(posedge dst_clk, negedge reset_n) begin
    if (~reset_n) begin
        dst_pulse <= 1'b0;
        prev_dst_level <= 1'b0;
    end
    else begin
        dst_pulse <= dst_level ^ prev_dst_level;
        prev_dst_level <= dst_level;
    end
end

endmodule
