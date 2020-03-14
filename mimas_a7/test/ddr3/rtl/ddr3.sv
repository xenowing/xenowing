`default_nettype none

module ddr3(
    input wire logic reset,
    input wire logic input_clk_100,

    inout wire logic [15:0] ddr3_dq,
    inout wire logic [1:0] ddr3_dqs_n,
    inout wire logic [1:0] ddr3_dqs_p,
    output wire logic [13:0] ddr3_addr,
    output wire logic [2:0] ddr3_ba,
    output wire logic ddr3_ras_n,
    output wire logic ddr3_cas_n,
    output wire logic ddr3_we_n,
    output wire logic ddr3_reset_n, // TODO: Add timing constraints for this pin
    output wire logic [0:0] ddr3_ck_p,
    output wire logic [0:0] ddr3_ck_n,
    output wire logic [0:0] ddr3_cke,
    output wire logic [0:0] ddr3_cs_n,
    output wire logic [1:0] ddr3_dm,
    output wire logic [0:0] ddr3_odt,

    output wire logic uart_tx,

    output wire logic led);

    logic sys_clk_200;
    logic clocking_locked;
    clk_mmcm clk_mmcm0(
        .clk_200(sys_clk_200),
        .reset(reset),
        .locked(clocking_locked),
        .clk_in1(input_clk_100));

    // TODO: Do we need this? Is this sufficient?
    logic sys_reset_n;
    reset_synchronizer reset_synchronizer0(
        .reset_n(~reset & clocking_locked),
        .clk(sys_clk_200),
        .reset_n_sync(sys_reset_n));

    logic ddr3_controller_calib_done;
    logic [27:0] app_addr;
    logic [2:0] app_cmd;
    logic app_en;
    logic [127:0] app_wdf_data;
    logic app_wdf_wren;
    logic [127:0] app_rd_data;
    logic app_rd_data_valid;
    logic app_rdy;
    logic app_wdf_rdy;
    logic [15:0] app_wdf_mask;
    logic clk_100;
    logic ddr3_controller_ui_clk_sync_rst;
    ddr3_controller ddr3_controller0(
        .sys_clk_i(sys_clk_200),
        .sys_rst(sys_reset_n),

        .ddr3_addr(ddr3_addr),
        .ddr3_ba(ddr3_ba),
        .ddr3_cas_n(ddr3_cas_n),
        .ddr3_ck_n(ddr3_ck_n),
        .ddr3_ck_p(ddr3_ck_p),
        .ddr3_cke(ddr3_cke),
        .ddr3_ras_n(ddr3_ras_n),
        .ddr3_reset_n(ddr3_reset_n),
        .ddr3_we_n(ddr3_we_n),
        .ddr3_dq(ddr3_dq),
        .ddr3_dqs_n(ddr3_dqs_n),
        .ddr3_dqs_p(ddr3_dqs_p),
        .ddr3_cs_n(ddr3_cs_n),
        .ddr3_dm(ddr3_dm),
        .ddr3_odt(ddr3_odt),

        .init_calib_complete(ddr3_controller_calib_done),

        .app_addr(app_addr),
        .app_cmd(app_cmd),
        .app_en(app_en),
        .app_wdf_data(app_wdf_data),
        .app_wdf_end(app_wdf_wren),
        .app_wdf_wren(app_wdf_wren),
        .app_rd_data(app_rd_data),
        .app_rd_data_valid(app_rd_data_valid),
        .app_rdy(app_rdy),
        .app_wdf_rdy(app_wdf_rdy),
        .app_sr_req(0),
        .app_ref_req(0),
        .app_zq_req(0),
        .app_wdf_mask(app_wdf_mask),

        .ui_clk(clk_100),
        .ui_clk_sync_rst(ddr3_controller_ui_clk_sync_rst));

    logic reset_n = ~ddr3_controller_ui_clk_sync_rst & ddr3_controller_calib_done;

    logic [7:0] uart_write_data;
    logic uart_write_req;
    logic uart_ready;
    uart_transmitter uart_transmitter0(
        .reset_n(reset_n),
        .clk(clk_100),

        .write_data(uart_write_data),
        .write_req(uart_write_req),
        .ready(uart_ready),

        .tx(uart_tx));

    localparam STATE_IDLE = 3'd0;
    localparam STATE_WRITE = 3'd1;
    localparam STATE_TRANSMIT_WRITE_CYCLES = 3'd2;
    localparam STATE_READ = 3'd3;
    localparam STATE_READ_WAIT = 3'd4;
    localparam STATE_TRANSMIT_READ_CYCLES = 3'd5;
    localparam STATE_PARK = 3'd6;
    localparam STATE_ERROR = 3'd7;
    logic [2:0] state;

    logic [31:0] word_counter;

    assign app_addr = {1'h0, word_counter[23:0], 3'h0};
    assign app_wdf_data = DATA_BASE + {96'h0, word_counter};

    localparam CMD_WRITE = 3'b000;
    localparam CMD_READ = 3'b001;

    localparam DATA_BASE = 128'hdeadbeefabad1deaba53b411fadebabe;
    localparam NUM_WORDS = 32'h1000000;

    logic [63:0] write_cycles;
    logic [63:0] read_cycles;
    logic [63:0] uart_write_word;
    logic [2:0] uart_write_byte_index;

    assign uart_write_data = uart_write_word[7:0];

    logic led_reg;
    assign led = led_reg;

    always_ff @(posedge clk_100) begin
        if (~reset_n) begin
            app_cmd <= 0;
            app_en <= 0;
            app_wdf_wren <= 0;
            app_wdf_mask <= 0;

            uart_write_req <= 0;

            state <= STATE_IDLE;

            word_counter <= 0;

            write_cycles <= 0;
            read_cycles <= 0;
            uart_write_word <= 0;
            uart_write_byte_index <= 0;

            led_reg <= 0;
        end
        else begin
            case (state)
                STATE_IDLE: begin
                    app_cmd <= CMD_WRITE;
                    app_en <= 1;
                    app_wdf_wren <= 1;

                    state <= STATE_WRITE;
                end

                STATE_WRITE: begin
                    // TODO: There's a lot of duped code here, but the idea is both the command and data write ports are actually
                    //  seperated, and either can be busy for any reason completely out of sync with the other. This means we need
                    //  to be careful in order to not issue extra commands or data in the case that one port is busy while the
                    //  other isn't. A simpler version of this code would be to have separate if statements to disable asserting
                    //  writes to each port separately and then a third to check if both ports have been written to, but this
                    //  introduces an additional wait cycle for _every write_, which literally _halves_ our performance, as in the
                    //  ideal case we should be able to issue a write almost every cycle. So, the lazy way to make this work is to
                    //  dupe the inner code like I've done here, but this can probably be done better.
                    if (app_rdy & app_en) begin
                        app_en <= 0;

                        if ((app_wdf_rdy & app_wdf_wren) | ~app_wdf_wren) begin
                            app_wdf_wren <= 0;

                            if (word_counter == NUM_WORDS - 1) begin
                                uart_write_req <= 1;

                                uart_write_word <= write_cycles;

                                state <= STATE_TRANSMIT_WRITE_CYCLES;
                            end
                            else begin
                                app_en <= 1;
                                app_wdf_wren <= 1;

                                word_counter <= word_counter + 32'h1;
                            end
                        end
                    end

                    if (app_wdf_rdy & app_wdf_wren) begin
                        app_wdf_wren <= 0;

                        if ((app_rdy & app_en) | ~app_en) begin
                            app_en <= 0;

                            if (word_counter == NUM_WORDS - 1) begin
                                uart_write_req <= 1;

                                uart_write_word <= write_cycles;

                                state <= STATE_TRANSMIT_WRITE_CYCLES;
                            end
                            else begin
                                app_en <= 1;
                                app_wdf_wren <= 1;

                                word_counter <= word_counter + 32'h1;
                            end
                        end
                    end

                    write_cycles <= write_cycles + 64'h1;
                end

                STATE_TRANSMIT_WRITE_CYCLES: begin
                    if (uart_ready) begin
                        if (uart_write_byte_index != 3'h7) begin
                            uart_write_word <= {8'h0, uart_write_word[63:8]};
                            uart_write_byte_index <= uart_write_byte_index + 3'h1;
                        end
                        else begin
                            app_cmd <= CMD_READ;
                            app_en <= 1;

                            uart_write_req <= 0;

                            state <= STATE_READ;

                            word_counter <= 0;
                        end
                    end
                end

                STATE_READ: begin
                    if (app_rdy) begin
                        if (word_counter == NUM_WORDS - 1) begin
                            app_en <= 0;

                            state <= STATE_READ_WAIT;
                        end
                        else begin
                            word_counter <= word_counter + 32'h1;
                        end
                    end

                    read_cycles <= read_cycles + 64'h1;
                end

                STATE_READ_WAIT: begin
                    if (read_check_done) begin
                        if (read_check_valid) begin
                            uart_write_req <= 1;

                            uart_write_word <= read_cycles;
                            uart_write_byte_index <= 0;

                            state <= STATE_TRANSMIT_READ_CYCLES;
                        end
                        else begin
                            state <= STATE_ERROR;
                        end
                    end

                    read_cycles <= read_cycles + 64'h1;
                end

                STATE_TRANSMIT_READ_CYCLES: begin
                    if (uart_ready) begin
                        if (uart_write_byte_index != 3'h7) begin
                            uart_write_word <= {8'h0, uart_write_word[63:8]};
                            uart_write_byte_index <= uart_write_byte_index + 3'h1;
                        end
                        else begin
                            uart_write_req <= 0;

                            state <= STATE_PARK;
                        end
                    end
                end

                STATE_PARK: begin
                    led_reg <= 1;
                end

                STATE_ERROR: begin
                    led_reg = ~led_reg;
                end

                default: begin
                    state <= STATE_ERROR;
                end
            endcase
        end
    end

    logic read_check_done;
    logic read_check_valid;
    logic [31:0] read_check_word_counter;

    always_ff @(posedge clk_100) begin
        if (~reset_n) begin
            read_check_done <= 0;
            read_check_valid <= 0;
            read_check_word_counter <= 0;
        end
        else begin
            if (~read_check_done & app_rd_data_valid) begin
                if (app_rd_data == DATA_BASE + {96'h0, read_check_word_counter}) begin
                    if (read_check_word_counter == NUM_WORDS - 1) begin
                        read_check_done <= 1;
                        read_check_valid <= 1;
                    end
                    else begin
                        read_check_word_counter <= read_check_word_counter + 32'h1;
                    end
                end
                else begin
                    read_check_done <= 1;
                    read_check_valid <= 0;
                end
            end
        end
    end

endmodule
