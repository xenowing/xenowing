`default_nettype none

module reset_synchronizer(
	input reset_n,
	input clk,

	output reset_n_sync);
	
	logic ff1, ff2;
	always_ff @(posedge clk or negedge reset_n) begin
		if (~reset_n) begin
			{ ff1, ff2 } <= 2'b0;
		end
		else begin
			{ ff2, ff1 } <= { ff1, 1'b1 };
		end
	end
	
	assign reset_n_sync = ff2;
	
endmodule
