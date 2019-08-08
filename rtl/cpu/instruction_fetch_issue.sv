`default_nettype none

module instruction_fetch_issue(
    input reset_n,
    input clk,

    output ready,
    input enable,

    input system_bus_ready,
    output [31:0] system_bus_addr,
    output [3:0] system_bus_byte_enable,
    output system_bus_read_req);

    logic [31:0] pc;
    logic [31:0] pc_next;

    assign ready = system_bus_ready;
    assign system_bus_addr = pc;
    assign system_bus_byte_enable = 4'b1111;
    assign system_bus_read_req = enable;

    always_comb begin
        pc_next = pc;

        if (enable && ready) begin
            pc_next = pc + 32'h4;
        end
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            pc <= 32'h10000000;
        end
        else begin
            pc <= pc_next;
        end
    end

endmodule
