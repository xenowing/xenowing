`default_nettype none

module core(
    input reset_n,
    input clk,

    input mem_ready,
    output logic [31:0] mem_addr,
    output logic [3:0] mem_byte_enable,
    output logic mem_read_req,
    input [31:0] mem_read_data,
    input mem_read_data_valid,

    input load_unit_read_ready,
    output logic load_unit_read_req,
    output logic [31:0] load_unit_read_addr,
    output logic [3:0] load_unit_read_byte_enable,
    input [31:0] load_unit_read_data,
    input load_unit_read_data_valid,

    input store_unit_write_ready,
    output logic store_unit_write_req,
    output logic [31:0] store_unit_write_addr,
    output logic [31:0] store_unit_write_data,
    output logic [3:0] store_unit_write_byte_enable);

    logic [31:0] mem_addr_next;
    logic [3:0] mem_byte_enable_next;
    logic mem_read_req_next;

    logic load_unit_read_req_next;
    logic [31:0] load_unit_read_addr_next;
    logic [3:0] load_unit_read_byte_enable_next;

    logic [31:0] load_data;
    logic [31:0] load_data_next;

    logic store_unit_write_req_next;
    logic [31:0] store_unit_write_addr_next;
    logic [31:0] store_unit_write_data_next;
    logic [3:0] store_unit_write_byte_enable_next;

    logic [31:0] pc;
    logic [31:0] pc_next;

    logic [31:0] rs1_value;
    logic [31:0] rs2_value;

    logic rd_write_enable;
    logic [31:0] rd_write_value;

    register_file register_file0(
        .clk(clk),
        .reset_n(reset_n),

        .read_addr1(rs1),
        .read_data1(rs1_value),

        .read_addr2(rs2),
        .read_data2(rs2_value),

        .write_enable(rd_write_enable),
        .write_addr(rd),
        .write_data(rd_write_value));

    logic [63:0] cycle;
    logic [63:0] cycle_next;
    logic [63:0] instret;
    logic [63:0] instret_next;

    localparam STATE_INITIAL_INSTRUCTION_FETCH_SETUP = 3'h0;
    localparam STATE_ERROR = 3'h1;
    localparam STATE_INSTRUCTION_FETCH = 3'h2;
    localparam STATE_INSTRUCTION_DECODE = 3'h3;
    localparam STATE_MEM_LOAD = 3'h4;
    localparam STATE_MEM_LOAD_WAIT = 3'h5;
    localparam STATE_MEM_STORE = 3'h6;
    localparam STATE_REG_WRITEBACK = 3'h7;
    logic [2:0] state;
    logic [2:0] state_next;

    alu_op_t alu_op;
    alu_op_t alu_op_next;
    logic [31:0] alu_lhs;
    logic [31:0] alu_lhs_next;
    logic [31:0] alu_rhs;
    logic [31:0] alu_rhs_next;
    logic [31:0] alu_res;

    logic [31:0] instruction;
    logic [31:0] instruction_next;

    logic [6:0] opcode;
    logic [4:0] rd;
    logic [4:0] rs1;
    logic [4:0] rs2;
    logic [2:0] funct3;
    logic [31:0] load_offset;
    logic [31:0] store_offset;
    logic [31:0] jump_offset;
    logic [31:0] branch_offset;
    logic [31:0] i_immediate;
    logic [31:0] u_immediate;
    logic [11:0] csr;
    assign opcode = instruction[6:0];
    assign rd = instruction[11:7];
    assign rs1 = instruction[19:15];
    assign rs2 = instruction[24:20];
    assign funct3 = instruction[14:12];
    assign load_offset = {{20{instruction[31]}}, instruction[31:20]};
    assign store_offset = {{20{instruction[31]}}, {instruction[31:25], instruction[11:7]}};
    assign jump_offset = {{11{instruction[31]}}, {instruction[31], instruction[19:12], instruction[20], instruction[30:21]}, 1'b0};
    assign branch_offset = {{19{instruction[31]}}, instruction[31], instruction[7], instruction[30:25], instruction[11:8], 1'b0};
    assign i_immediate = {{20{instruction[31]}}, instruction[31:20]};
    assign u_immediate = {instruction[31:12], 12'h0};
    assign csr = instruction[31:20];

    alu alu0(
        .op(alu_op),
        .lhs(alu_lhs),
        .rhs(alu_rhs),
        .res(alu_res));

    always_comb begin
        mem_addr_next = mem_addr;
        mem_byte_enable_next = mem_byte_enable;
        mem_read_req_next = mem_read_req;

        load_unit_read_req_next = load_unit_read_req;
        load_unit_read_addr_next = load_unit_read_addr;
        load_unit_read_byte_enable_next = load_unit_read_byte_enable;

        load_data_next = load_data;

        store_unit_write_req_next = store_unit_write_req;
        store_unit_write_addr_next = store_unit_write_addr;
        store_unit_write_data_next = store_unit_write_data;
        store_unit_write_byte_enable_next = store_unit_write_byte_enable;

        pc_next = pc;

        rd_write_enable = 0;
        rd_write_value = 32'h0;

        cycle_next = cycle;
        instret_next = instret;

        state_next = state;

        alu_op_next = alu_op;
        alu_lhs_next = alu_lhs;
        alu_rhs_next = alu_rhs;

        instruction_next = instruction;

        cycle_next = cycle + 64'h1;

        case (state)
            STATE_INITIAL_INSTRUCTION_FETCH_SETUP: begin
                // Set up instruction fetch state
                mem_addr_next = pc;
                mem_byte_enable_next = 4'hf;
                mem_read_req_next = 1;

                state_next = STATE_INSTRUCTION_FETCH;
            end

            STATE_ERROR: begin
                // TODO
            end

            STATE_INSTRUCTION_FETCH: begin
                // Finish asserting fetch read
                if (mem_ready) begin
                    mem_read_req_next = 0;
                end

                if (!mem_read_req_next && mem_read_data_valid) begin
                    instruction_next = mem_read_data;

                    state_next = STATE_INSTRUCTION_DECODE;
                end
            end

            STATE_INSTRUCTION_DECODE: begin
                state_next = STATE_REG_WRITEBACK;

                case (funct3)
                    3'b000: alu_op_next = !instruction[30] ? ADD : SUB;
                    3'b001: alu_op_next = SLL;
                    3'b010: alu_op_next = LT;
                    3'b011: alu_op_next = LTU;
                    3'b100: alu_op_next = XOR;
                    3'b101: alu_op_next = !instruction[30] ? SRL : SRA;
                    3'b110: alu_op_next = OR;
                    3'b111: alu_op_next = AND;
                endcase

                alu_lhs_next = rs1_value;
                alu_rhs_next = rs2_value;

                case (opcode)
                    7'b0110111: begin
                        // lui
                        alu_op_next = ADD;
                        alu_lhs_next = u_immediate;
                    end
                    7'b0010111: begin
                        // auipc
                        alu_op_next = ADD;
                        alu_lhs_next = u_immediate;
                        alu_rhs_next = pc;
                    end
                    7'b1101111: begin
                        // jal
                        alu_op_next = ADD;
                        alu_lhs_next = jump_offset;
                        alu_rhs_next = pc;
                    end
                    7'b1100111: begin
                        // jalr
                        alu_op_next = ADD;
                        alu_rhs_next = i_immediate;
                    end
                    7'b1100011: begin
                        // branches
                        case (funct3)
                            3'b000: alu_op_next = EQ;
                            3'b001: alu_op_next = NE;
                            3'b100: alu_op_next = LT;
                            3'b101: alu_op_next = GE;
                            3'b110: alu_op_next = LTU;
                            3'b111: alu_op_next = GEU;
                            default: state_next = STATE_ERROR;
                        endcase
                    end
                    7'b0000011: begin
                        // loads
                        state_next = STATE_MEM_LOAD;

                        alu_op_next = ADD;
                        alu_rhs_next = load_offset;
                    end
                    7'b0100011: begin
                        // stores
                        state_next = STATE_MEM_STORE;

                        alu_op_next = ADD;
                        alu_rhs_next = store_offset;
                    end
                    7'b0010011: begin
                        // immediate computation
                        //  default alu rhs should be immediate, except for shifts,
                        //  which use rs2 (directly, not its register value)
                        alu_rhs_next = (funct3 == 3'b001 || funct3 == 3'b101) ? {27'h0, rs2} : i_immediate;
                    end
                    7'b0110011: begin
                        // register computation
                        //  default alu lhs/rhs are already correct; do nothing
                    end
                    7'b0001111: begin
                        // fences (do nothing)
                        //  Note that if we introduce an icache later, fence.i should flush it
                    end
                    7'b1110011: begin
                        // system instr's
                        case (funct3)
                            3'b000: begin
                                // ecall/ebreak (do nothing)
                            end
                            3'b001, 3'b010, 3'b011, 3'b101, 3'b110, 3'b111: begin
                                // csrrw, csrrs, csrrc, csrrwi, csrrsi, csrrci (do nothing)
                            end
                            default: state_next = STATE_ERROR;
                        endcase
                    end
                    default: state_next = STATE_ERROR;
                endcase
            end

            STATE_MEM_LOAD: begin
                state_next = STATE_MEM_LOAD_WAIT;

                load_unit_read_req_next = 1;
                load_unit_read_addr_next = alu_res;

                case (funct3)
                    3'b000, 3'b100: load_unit_read_byte_enable_next = 4'b0001; // lb, lbu
                    3'b001, 3'b101: load_unit_read_byte_enable_next = 4'b0011; // lh, lhu
                    3'b010: load_unit_read_byte_enable_next = 4'b1111; // lw
                endcase
            end

            STATE_MEM_LOAD_WAIT: begin
                if (load_unit_read_ready) begin
                    load_unit_read_req_next = 0;

                    if (load_unit_read_data_valid) begin
                        load_data_next = load_unit_read_data;

                        state_next = STATE_REG_WRITEBACK;
                    end
                end
            end

            STATE_MEM_STORE: begin
                state_next = STATE_REG_WRITEBACK;

                store_unit_write_req_next = 1;
                store_unit_write_addr_next = alu_res;
                store_unit_write_data_next = rs2_value;

                case (funct3)
                    3'b000: store_unit_write_byte_enable_next = 4'b0001; // sb
                    3'b001: store_unit_write_byte_enable_next = 4'b0011; // sh
                    3'b010: store_unit_write_byte_enable_next = 4'b1111; // sw
                endcase
            end

            STATE_REG_WRITEBACK: begin
                // Finish asserting write, if any
                if (store_unit_write_ready) begin
                    store_unit_write_req_next = 0;
                end

                if (!store_unit_write_req_next) begin
                    state_next = STATE_INSTRUCTION_FETCH;
                    pc_next = pc + 32'h4;

                    case (opcode)
                        7'b0110111, 7'b0010111, 7'b0010011, 7'b0110011: begin
                            // lui, auipc, immediate computation, register computation
                            rd_write_enable = 1;
                            rd_write_value = alu_res;
                        end
                        7'b1101111: begin
                            // jal
                            rd_write_enable = 1;
                            rd_write_value = pc + 32'h4;

                            pc_next = alu_res;
                        end
                        7'b1100111: begin
                            // jalr
                            rd_write_enable = 1;
                            rd_write_value = pc + 32'h4;

                            pc_next = {alu_res[31:1], 1'b0};
                        end
                        7'b1100011: begin
                            // branches
                            if (alu_res[0]) begin
                                pc_next = pc + branch_offset;
                            end
                        end
                        7'b0000011: begin
                            // loads
                            rd_write_enable = 1;
                            case (funct3)
                                3'b000: begin
                                    // lb
                                    rd_write_value = {{24{load_data[7]}}, load_data[7:0]};
                                end
                                3'b100: begin
                                    // lbu
                                    rd_write_value = {24'b0, load_data[7:0]};
                                end
                                3'b001: begin
                                    // lh
                                    rd_write_value = {{16{load_data[15]}}, load_data[15:0]};
                                end
                                3'b101: begin
                                    // lhu
                                    rd_write_value = {16'b0, load_data[15:0]};
                                end
                                3'b010: begin
                                    // lw
                                    rd_write_value = load_data;
                                end
                            endcase
                        end
                        7'b1110011: begin
                            // system instr's
                            case (funct3)
                                3'b001, 3'b010, 3'b011, 3'b101, 3'b110, 3'b111: begin
                                    // csrrw, csrrs, csrrc, csrrwi, csrrsi, csrrci
                                    rd_write_enable = 1;
                                    case (csr)
                                        12'hc00, 12'hc01: rd_write_value = cycle[31:0]; // cycle, time
                                        12'hc02: rd_write_value = instret[31:0]; // instret
                                        12'hc80, 12'hc81: rd_write_value = cycle[63:32]; // cycleh, timeh
                                        12'hc82: rd_write_value = instret[63:32]; // instreth
                                        default: state_next = STATE_ERROR;
                                    endcase
                                end
                            endcase
                        end
                    endcase

                    // Set up instruction fetch state
                    if (pc_next[1:0] != 2'b0) begin
                        // Misaligned PC
                        state_next = STATE_ERROR;
                    end
                    else begin
                        mem_addr_next = pc_next;
                        mem_byte_enable_next = 4'hf;
                        mem_read_req_next = 1;

                        instret_next = instret + 64'h1;
                    end
                end
            end

            default: state_next = STATE_ERROR;
        endcase
    end

    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            mem_addr <= 32'h0;
            mem_byte_enable <= 4'h0;
            mem_read_req <= 0;

            load_unit_read_req <= 0;
            load_unit_read_addr <= 32'h0;
            load_unit_read_byte_enable <= 4'h0;

            load_data <= 32'h0;

            store_unit_write_req <= 0;
            store_unit_write_addr <= 32'h0;
            store_unit_write_data <= 32'h0;
            store_unit_write_byte_enable <= 4'h0;

            pc <= 32'h10000000;

            cycle <= 64'h0;
            instret <= 64'h0;

            state <= STATE_INITIAL_INSTRUCTION_FETCH_SETUP;

            alu_op <= ADD;
            alu_lhs <= 32'h0;
            alu_rhs <= 32'h0;

            instruction <= 32'h0;
        end
        else begin
            mem_addr <= mem_addr_next;
            mem_byte_enable <= mem_byte_enable_next;
            mem_read_req <= mem_read_req_next;

            load_unit_read_req <= load_unit_read_req_next;
            load_unit_read_addr <= load_unit_read_addr_next;
            load_unit_read_byte_enable <= load_unit_read_byte_enable_next;

            load_data <= load_data_next;

            store_unit_write_req <= store_unit_write_req_next;
            store_unit_write_addr <= store_unit_write_addr_next;
            store_unit_write_data <= store_unit_write_data_next;
            store_unit_write_byte_enable <= store_unit_write_byte_enable_next;

            pc <= pc_next;

            cycle <= cycle_next;
            instret <= instret_next;

            state <= state_next;

            alu_op <= alu_op_next;
            alu_lhs <= alu_lhs_next;
            alu_rhs <= alu_rhs_next;

            instruction <= instruction_next;
        end
    end

endmodule
