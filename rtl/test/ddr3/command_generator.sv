module command_generator(
    input reset_n,
    input clk,

    input avl_ready,
    output logic avl_burstbegin,
    output logic [23:0] avl_addr,
    output logic [63:0] avl_wdata,
    output logic [7:0] avl_be,
    output logic avl_read_req,
    output logic avl_write_req,
    output logic [6:0] avl_size,

    input ddr3_init_done,
    input ddr3_cal_success,
    input ddr3_cal_fail,

    output logic is_finished,
    output logic pass,
    output logic fail);

    logic avl_burstbegin_next;
    logic [23:0] avl_addr_next;
    logic [63:0] avl_wdata_next;
    logic [7:0] avl_be_next;
    logic avl_read_req_next;
    logic avl_write_req_next;
    logic [6:0] avl_size_next;

    logic is_finished_next;
    logic pass_next;
    logic fail_next;

    localparam STATE_WAIT_FOR_INIT = 3'h0;
    localparam STATE_ERROR = 3'h1;
    localparam STATE_WRITE_BYTES = 3'h2;
    localparam STATE_READ_BYTES = 3'h3;
    localparam STATE_FINISH_CYCLE = 3'h4;
    localparam STATE_FINISHED = 3'h5;
    logic [2:0] state;
    logic [2:0] state_next;

    logic [24:0] test_counter;
    logic [24:0] test_counter_next;

    always_comb begin
        avl_burstbegin_next = avl_burstbegin;
        avl_addr_next = avl_addr;
        avl_wdata_next = avl_wdata;
        avl_be_next = avl_be;
        avl_read_req_next = avl_read_req;
        avl_write_req_next = avl_write_req;
        avl_size_next = avl_size;

        is_finished_next = is_finished;
        pass_next = pass;
        fail_next = fail;

        state_next = state;

        test_counter_next = test_counter;

        case (state)
            STATE_WAIT_FOR_INIT: begin
                if (ddr3_init_done) begin
                    if (ddr3_cal_success) begin
                        state_next = STATE_WRITE_BYTES;
                    end
                    else if (ddr3_cal_fail) begin
                        state_next = STATE_ERROR;
                    end
                end
            end

            STATE_ERROR: begin
                is_finished_next = 1;
                fail_next = 1;
            end

            STATE_WRITE_BYTES: begin
                // Write 12 bytes
                avl_burstbegin_next = 1;
                avl_addr_next = test_counter[23:0];
                avl_wdata_next = 64'hdeadfadebabebeef ^ {39'h0, test_counter};
                avl_be_next = 8'hff;
                avl_write_req_next = 1;
                avl_size_next = 7'h1;

                state_next = STATE_READ_BYTES;
            end

            STATE_READ_BYTES: begin
                avl_burstbegin_next = 0;

                if (avl_ready) begin
                    avl_write_req_next = 0;

                    // Read 12 bytes
                    avl_burstbegin_next = 1;
                    avl_addr_next = test_counter[23:0];
                    avl_be_next = 8'hff;
                    avl_read_req_next = 1;
                    avl_size_next = 7'h1;

                    state_next = STATE_FINISH_CYCLE;
                end
            end

            STATE_FINISH_CYCLE: begin
                avl_burstbegin_next = 0;

                if (avl_ready) begin
                    avl_read_req_next = 0;

                    if (test_counter[24]) begin
                        state_next = STATE_FINISHED;
                    end
                    else begin
                        test_counter_next = test_counter + 25'h1;

                        state_next = STATE_WRITE_BYTES;
                    end
                end
            end

            STATE_FINISHED: begin
                is_finished_next = 1;
                pass_next = 1;
            end

            default: begin
                state_next = STATE_ERROR;
            end
        endcase
    end

    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            avl_burstbegin <= 0;
            avl_addr <= 24'h0;
            avl_wdata <= 64'h0;
            avl_be <= 8'h0;
            avl_read_req <= 0;
            avl_write_req <= 0;
            avl_size <= 7'h0;

            is_finished <= 0;
            pass <= 0;
            fail <= 0;

            state <= STATE_WAIT_FOR_INIT;

            test_counter <= 25'h0;
        end
        else begin
            avl_burstbegin <= avl_burstbegin_next;
            avl_addr <= avl_addr_next;
            avl_wdata <= avl_wdata_next;
            avl_be <= avl_be_next;
            avl_read_req <= avl_read_req_next;
            avl_write_req <= avl_write_req_next;
            avl_size <= avl_size_next;

            is_finished <= is_finished_next;
            pass <= pass_next;
            fail <= fail_next;

            state <= state_next;

            test_counter <= test_counter_next;
        end
    end

endmodule
