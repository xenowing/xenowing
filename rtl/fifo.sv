`default_nettype none

module fifo#(parameter data_width = 32, depth_bits = 4)(
    input reset_n,
    input clk,

    output empty,
    output full,

    input [data_width - 1:0] write_data,
    input write_enable,

    output [data_width - 1:0] read_data,
    input read_enable);

    localparam depth = 1 << depth_bits;

    logic [data_width - 1:0] mem[0:depth - 1];

    logic [depth_bits - 1:0] write_addr;
    logic [depth_bits - 1:0] write_addr_next;
    logic [depth_bits - 1:0] read_addr;
    logic [depth_bits - 1:0] read_addr_next;

    logic [depth_bits:0] count;
    logic [depth_bits:0] count_next;

    assign empty = count == 0;
    assign full = count == depth;

    assign read_data = mem[read_addr];

    always_comb begin
        write_addr_next = write_addr;
        read_addr_next = read_addr;

        count_next = count;

        if (write_enable) begin
            write_addr_next = write_addr + 1;
            count_next = count_next + 1;
        end

        if (read_enable) begin
            read_addr_next = read_addr + 1;
            count_next = count_next - 1;
        end
    end

    always @(posedge clk) begin
        if (!reset_n) begin
            write_addr <= 0;
            read_addr <= 0;

            count <= 0;
        end
        else begin
            if (write_enable) begin
                mem[write_data] <= write_data;
            end

            write_addr <= write_addr_next;
            read_addr <= read_addr_next;

            count <= count_next;
        end
    end

endmodule
