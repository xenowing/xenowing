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

    output wire logic success_led,
    output wire logic init_calib_complete_led,
    output wire logic error_led);

    logic sys_clk_200;
    logic clocking_locked;
    clk_mmcm clk_mmcm0(
        .clk_200(sys_clk_200),
        .reset(reset),
        .locked(clocking_locked),
        .clk_in1(input_clk_100));

    logic sys_reset_n;
    SyncChain #(.DEFAULT(1'b0)) reset_sync_chain(
        .reset_n(~reset),
        .clk(sys_clk_200),

        .x(1'b1),

        .x_sync(sys_reset_n));

    logic init_calib_complete;
    logic [27:0] app_addr;
    logic [2:0] app_cmd;
    logic app_en;
    logic [127:0] app_wdf_data;
    logic app_wdf_wren;
    logic app_wdf_end;
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

        .init_calib_complete(init_calib_complete),

        .app_addr(app_addr),
        .app_cmd(app_cmd),
        .app_en(app_en),
        .app_wdf_data(app_wdf_data),
        .app_wdf_end(app_wdf_end),
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

    logic reset_n = ~ddr3_controller_ui_clk_sync_rst & init_calib_complete;

    logic bus_enable;
    logic [23:0] bus_addr;
    logic bus_write;
    logic [127:0] bus_write_data;
    logic [15:0] bus_write_byte_enable;
    logic bus_ready;
    logic [127:0] bus_read_data;
    logic bus_read_data_valid;
    logic [23:0] bridge_app_addr;
    BusterMigUiBridge buster_mig_ui_bridge(
        .reset_n(reset_n),
        .clk(clk_100),

        .bus_enable(bus_enable),
        .bus_addr(bus_addr),
        .bus_write(bus_write),
        .bus_write_data(bus_write_data),
        .bus_write_byte_enable(bus_write_byte_enable),
        .bus_ready(bus_ready),
        .bus_read_data(bus_read_data),
        .bus_read_data_valid(bus_read_data_valid),

        .init_calib_complete(init_calib_complete),

        .app_rdy(app_rdy),
        .app_en(app_en),
        .app_cmd(app_cmd),
        .app_addr(bridge_app_addr),

        .app_wdf_rdy(app_wdf_rdy),
        .app_wdf_data(app_wdf_data),
        .app_wdf_wren(app_wdf_wren),
        .app_wdf_mask(app_wdf_mask),
        .app_wdf_end(app_wdf_end),

        .app_rd_data(app_rd_data),
        .app_rd_data_valid(app_rd_data_valid));

    logic [7:0] uart_tx_data;
    logic uart_tx_enable;
    logic uart_tx_ready;
    UartTx uart_tx0(
        .reset_n(reset_n),
        .clk(clk_100),

        .data(uart_tx_data),
        .enable(uart_tx_enable),
        .ready(uart_tx_ready),

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

    localparam DATA_BASE = 128'hdeadbeefabad1deaba53b411fadebabe;
    localparam NUM_WORDS = 32'h1000000;

    assign app_addr = {1'h0, bridge_app_addr, 3'h0};

    assign bus_addr = word_counter[23:0];
    assign bus_write_data = DATA_BASE + {96'h0, word_counter};

    logic [63:0] write_cycles;
    logic [63:0] read_cycles;
    logic [63:0] uart_write_word;
    logic [2:0] uart_write_byte_index;

    assign uart_tx_data = uart_write_word[7:0];

    logic success_led_reg;
    assign success_led = success_led_reg;

    assign init_calib_complete_led = init_calib_complete;

    logic error_led_reg;
    assign error_led = error_led_reg;

    always_ff @(posedge clk_100) begin
        if (~reset_n) begin
            bus_enable <= 0;
            bus_write <= 0;
            bus_write_byte_enable <= 16'h0000;

            uart_tx_enable <= 0;

            state <= STATE_IDLE;

            word_counter <= 0;

            write_cycles <= 0;
            read_cycles <= 0;
            uart_write_word <= 0;
            uart_write_byte_index <= 0;

            success_led_reg <= 0;
            error_led_reg <= 0;
        end
        else begin
            case (state)
                STATE_IDLE: begin
                    bus_enable <= 1;
                    bus_write <= 1;
                    bus_write_byte_enable <= 16'hffff;

                    state <= STATE_WRITE;
                end

                STATE_WRITE: begin
                    if (bus_enable & bus_ready) begin
                        if (word_counter == NUM_WORDS - 1) begin
                            bus_enable <= 0;

                            uart_tx_enable <= 1;

                            uart_write_word <= write_cycles;

                            state <= STATE_TRANSMIT_WRITE_CYCLES;
                        end
                        else begin
                            word_counter <= word_counter + 32'h1;
                        end
                    end

                    write_cycles <= write_cycles + 64'h1;
                end

                STATE_TRANSMIT_WRITE_CYCLES: begin
                    if (uart_tx_ready) begin
                        if (uart_write_byte_index != 3'h7) begin
                            uart_write_word <= {8'h0, uart_write_word[63:8]};
                            uart_write_byte_index <= uart_write_byte_index + 3'h1;
                        end
                        else begin
                            bus_enable <= 1;
                            bus_write <= 0;

                            uart_tx_enable <= 0;

                            state <= STATE_READ;

                            word_counter <= 0;
                        end
                    end
                end

                STATE_READ: begin
                    if (bus_ready) begin
                        if (word_counter == NUM_WORDS - 1) begin
                            bus_enable <= 0;

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
                            uart_tx_enable <= 1;

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
                    if (uart_tx_ready) begin
                        if (uart_write_byte_index != 3'h7) begin
                            uart_write_word <= {8'h0, uart_write_word[63:8]};
                            uart_write_byte_index <= uart_write_byte_index + 3'h1;
                        end
                        else begin
                            uart_tx_enable <= 0;

                            state <= STATE_PARK;
                        end
                    end
                end

                STATE_PARK: begin
                    success_led_reg <= 1;
                end

                STATE_ERROR: begin
                    error_led_reg <= 1;
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
            if (~read_check_done & bus_read_data_valid) begin
                if (bus_read_data == DATA_BASE + {96'h0, read_check_word_counter}) begin
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
