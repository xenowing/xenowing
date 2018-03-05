`default_nettype none

module uart_receiver(
    input reset_n,
    input clk,

    output logic [7:0] data,
    output logic data_ready,

    input rx);

    logic [7:0] data_next;
    logic data_ready_next;

    // Metastability prevention ff's for raw rx signal
    logic rx_ms_1;
    logic rx_ms_2;
    logic rx_stable;

    logic double_baud_clk_soft_reset;
    logic double_baud_clk_soft_reset_next;

    logic double_baud_clk_edge;
    uart_clock_divider #(.baud_rate(115200 * 2)) uart_clock_divider0(
        .reset_n(reset_n & !double_baud_clk_soft_reset),
        .clk(clk),

        .clock_edge(double_baud_clk_edge));

    localparam STATE_IDLE = 3'h0;
    localparam STATE_START_BIT = 3'h1;
    localparam STATE_BIT = 3'h2;
    localparam STATE_STOP_BIT = 3'h3;
    localparam STATE_WAIT_FOR_EOT = 3'h4;
    logic [2:0] state;
    logic [2:0] state_next;

    logic [1:0] double_baud_clk_tick_counter;
    logic [1:0] double_baud_clk_tick_counter_next;

    logic [2:0] bit_receive_index;
    logic [2:0] bit_receive_index_next;

    always_comb begin
        data_next = data;
        data_ready_next = data_ready;

        double_baud_clk_soft_reset_next = double_baud_clk_soft_reset;

        state_next = state;

        double_baud_clk_tick_counter_next = double_baud_clk_tick_counter;

        bit_receive_index_next = bit_receive_index;

        data_ready_next = 0;

        double_baud_clk_soft_reset_next = 0;

        if (double_baud_clk_edge) begin
            double_baud_clk_tick_counter_next = double_baud_clk_tick_counter + 2'h1;
        end    

        case (state)
            STATE_IDLE: begin
                if (!rx_stable) begin
                    state_next = STATE_START_BIT;
                    double_baud_clk_soft_reset_next = 1;
                    double_baud_clk_tick_counter_next = 2'h0;
                end
            end

            STATE_START_BIT: begin
                if (double_baud_clk_tick_counter == 2'h1) begin
                    if (!rx_stable) begin
                        state_next = STATE_BIT;
                        bit_receive_index_next = 3'h0;
                        double_baud_clk_tick_counter_next = 2'h0;
                    end
                    else begin
                        state_next = STATE_IDLE;
                    end
                end
            end

            STATE_BIT: begin
                if (double_baud_clk_tick_counter == 2'h2) begin
                    data_next[bit_receive_index] = rx_stable;

                    if (bit_receive_index == 3'h7) begin
                        state_next = STATE_STOP_BIT;
                    end
                    else begin
                        bit_receive_index_next = bit_receive_index + 3'h1;
                    end

                    double_baud_clk_tick_counter_next = 2'h0;
                end
            end

            STATE_STOP_BIT: begin
                if (double_baud_clk_tick_counter == 2'h2) begin
                    state_next = STATE_WAIT_FOR_EOT;
                end
            end

            STATE_WAIT_FOR_EOT: begin
                if (rx_stable == 1) begin
                    state_next = STATE_IDLE;
                    data_ready_next = 1;
                end
            end

            default: state_next = STATE_IDLE;
        endcase
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            data <= 8'h0;
            data_ready <= 0;

            rx_ms_1 <= 1;
            rx_ms_2 <= 1;
            rx_stable <= 1;

            double_baud_clk_soft_reset <= 0;

            state <= STATE_IDLE;

            double_baud_clk_tick_counter <= 2'h0;

            bit_receive_index <= 3'h0;
        end
        else begin
            data <= data_next;
            data_ready <= data_ready_next;

            rx_ms_1 <= rx;
            rx_ms_2 <= rx_ms_1;
            rx_stable <= rx_ms_2;

            double_baud_clk_soft_reset <= double_baud_clk_soft_reset_next;

            state <= state_next;

            double_baud_clk_tick_counter <= double_baud_clk_tick_counter_next;

            bit_receive_index <= bit_receive_index_next;
        end
    end

endmodule
