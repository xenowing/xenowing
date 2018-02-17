module led_interface(
    input reset_n,
    input clk,

    input [31:0] write_data,
    input [3:0] byte_enable,
    input write_req,
    input read_req,
    output [31:0] read_data,
    output logic read_data_valid,

    output logic leds[2:0]);

    assign read_data = {29'h0, leds};
    logic read_data_valid_next;

    logic [2:0] leds_next;

    always_comb begin
        read_data_valid_next = read_data_valid;

        leds_next = leds;

        read_data_valid_next = read_req;

        if (write_req && byte_enable[0]) begin
            leds_next = write_data[2:0];
        end
    end

    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            read_data_valid <= 0;

            leds <= 3'h0;
        end
        else begin
            read_data_valid <= read_data_valid_next;

            leds <= leds_next;
        end
    end

endmodule
