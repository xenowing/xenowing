module read_buffer(
    input reset_n,
    input clk,

    input clear,

    input [31:0] read_data,
    input read_data_valid,

    output logic [31:0] data[0:1],
    output logic [1:0] count);

    logic [31:0] data_next[0:1];
    logic [1:0] count_next;

    always_comb begin
        data_next = data;
        count_next = count;

        if (clear) begin
            count_next = 2'h0;
        end

        if (read_data_valid) begin
            data_next[count[0]] = read_data;
            count_next = count + 2'h1;
        end
    end

    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            data <= '{default:0};
            count <= 2'h0;
        end
        else begin
            data <= data_next;
            count <= count_next;
        end
    end

endmodule
