`default_nettype none

module uart_transmitter(
    input wire reset_n,
    input wire clk,

    input wire [7:0] write_data,
    input wire write_req,
    output wire ready,

    output logic tx);

    logic tx_next;

    localparam STATE_IDLE = 2'h0;
    localparam STATE_START_BIT = 2'h1;
    localparam STATE_BIT = 2'h2;
    localparam STATE_STOP_BIT = 2'h3;
    logic [1:0] state;
    logic [1:0] state_next;

    assign ready = state == STATE_IDLE;

    logic [7:0] write_data_latch;
    logic [7:0] write_data_latch_next;

    logic baud_clk_soft_reset;
    logic baud_clk_soft_reset_next;

    logic baud_clk_edge;
    uart_clock_divider #(.baud_rate(115200)) uart_clock_divider0(
        .reset_n(reset_n & !baud_clk_soft_reset),
        .clk(clk),

        .clock_edge(baud_clk_edge));

    logic [2:0] bit_send_index;
    logic [2:0] bit_send_index_next;

    always_comb begin
        tx_next = tx;

        state_next = state;

        write_data_latch_next = write_data_latch;

        baud_clk_soft_reset_next = baud_clk_soft_reset;

        bit_send_index_next = bit_send_index;

        baud_clk_soft_reset_next = 0;

        case (state)
            STATE_IDLE: begin
                if (write_req) begin
                    state_next = STATE_START_BIT;
                    write_data_latch_next = write_data;
                    baud_clk_soft_reset_next = 1;
                end
            end

            STATE_START_BIT: begin
                tx_next = 0;

                if (baud_clk_edge) begin
                    state_next = STATE_BIT;
                    bit_send_index_next = 0;
                end
            end

            STATE_BIT: begin
                tx_next = write_data_latch[bit_send_index];

                if (baud_clk_edge) begin
                    if (bit_send_index == 3'h7) begin
                        state_next = STATE_STOP_BIT;
                    end
                    else begin
                        bit_send_index_next = bit_send_index + 3'h1;
                    end
                end
            end

            STATE_STOP_BIT: begin
                tx_next = 1;

                if (baud_clk_edge) begin
                    state_next = STATE_IDLE;
                end
            end
        endcase
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            tx <= 1;

            state <= STATE_IDLE;

            write_data_latch <= 8'h0;

            baud_clk_soft_reset <= 0;

            bit_send_index <= 3'h0;
        end
        else begin
            tx <= tx_next;

            state <= state_next;

            write_data_latch <= write_data_latch_next;

            baud_clk_soft_reset <= baud_clk_soft_reset_next;

            bit_send_index <= bit_send_index_next;
        end
    end

endmodule
