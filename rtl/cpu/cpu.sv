module cpu(
    input reset_n,
    input clk,

    input ready,
    output logic [31:0] addr,
    output logic [31:0] write_data,
    output logic [3:0] byte_enable,
    output logic write_req,
    output logic read_req,
    input [31:0] read_data,
    input read_data_valid);

    logic [31:0] addr_next;
    logic [31:0] write_data_next;
    logic [3:0] byte_enable_next;
    logic write_req_next;
    logic read_req_next;

    logic [31:0] pc;
    logic [31:0] pc_next;

    logic [31:0] regs[0:30];
    logic [31:0] regs_next[0:30];

    logic [63:0] cycle;
    logic [63:0] cycle_next;
    logic [63:0] instret;
    logic [63:0] instret_next;

    localparam STATE_INITIAL_INSTRUCTION_FETCH_SETUP = 4'h0;
    localparam STATE_ERROR = 4'h1;
    localparam STATE_INSTRUCTION_FETCH = 4'h2;
    localparam STATE_INSTRUCTION_DECODE = 4'h3;
    localparam STATE_MEM_LOAD = 4'h4;
    localparam STATE_MEM_LOAD_2 = 4'h5;
    localparam STATE_MEM_LOAD_WAIT = 4'h6;
    localparam STATE_MEM_STORE = 4'h7;
    localparam STATE_MEM_STORE_2 = 4'h8;
    localparam STATE_REG_WRITEBACK = 4'h9;
    logic [3:0] state;
    logic [3:0] state_next;

    alu_op_t alu_op;
    alu_op_t alu_op_next;
    logic [31:0] alu_lhs;
    logic [31:0] alu_lhs_next;
    logic [31:0] alu_rhs;
    logic [31:0] alu_rhs_next;
    logic [31:0] alu_res;

    logic read_buffer_clear;
    logic read_buffer_clear_next;

    logic [31:0] instruction;
    logic [31:0] instruction_next;

    logic [6:0] opcode;
    logic [4:0] rd;
    logic [4:0] rs1;
    logic [4:0] rs2;
    logic [31:0] rs1_value;
    logic [31:0] rs2_value;
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
    assign rs1_value = rs1 > 0 ? regs[rs1 - 1] : 32'h0;
    assign rs2_value = rs2 > 0 ? regs[rs2 - 1] : 32'h0;
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

    logic [31:0] read_buffer_data[0:3];
    logic [1:0] read_buffer_count;
    read_buffer read_buffer0(
        .clk(clk),
        .reset_n(reset_n),
        .clear(read_buffer_clear),
        .read_data(read_data),
        .read_data_valid(read_data_valid),
        .data(read_buffer_data),
        .count(read_buffer_count));

    always_comb begin
        addr_next = addr;
        write_data_next = write_data;
        byte_enable_next = byte_enable;
        write_req_next = write_req;
        read_req_next = read_req;

        pc_next = pc;

        regs_next = regs;

        cycle_next = cycle;
        instret_next = instret;

        state_next = state;

        alu_op_next = alu_op;
        alu_lhs_next = alu_lhs;
        alu_rhs_next = alu_rhs;

        read_buffer_clear_next = read_buffer_clear;

        instruction_next = instruction;

        cycle_next = cycle + 64'h1;

        read_buffer_clear_next = 0;

        case (state)
            STATE_INITIAL_INSTRUCTION_FETCH_SETUP: begin
                // Set up instruction fetch state
                addr_next = pc;
                byte_enable_next = 4'hf;
                read_req_next = 1;

                state_next = STATE_INSTRUCTION_FETCH;
            end

            STATE_ERROR: begin
                // TODO
            end

            STATE_INSTRUCTION_FETCH: begin
                if (ready) begin
                    // Finish asserting read
                    read_req_next = 0;

                    if (read_data_valid) begin
                        instruction_next = read_data;

                        state_next = STATE_INSTRUCTION_DECODE;
                    end
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

                        read_buffer_clear_next = 1;
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

                addr_next = {alu_res[31:2], 2'b0};
                read_req_next = 1;

                case (funct3)
                    3'b000, 3'b100: begin
                        // lb, lbu
                        byte_enable_next = 4'b0001 << alu_res[1:0];
                    end
                    3'b001, 3'b101: begin
                        // lh, lhu
                        byte_enable_next = 4'b0011 << alu_res[1:0];
                        if (alu_res[1:0] == 2'b11) begin
                            state_next = STATE_MEM_LOAD_2;
                        end
                    end
                    3'b010: begin
                        // lw
                        byte_enable_next = 4'b1111 << alu_res[1:0];
                        if (alu_res[1:0] != 2'b00) begin
                            state_next = STATE_MEM_LOAD_2;
                        end
                    end
                    default: state_next = STATE_ERROR;
                endcase
            end

            STATE_MEM_LOAD_2: begin
                if (ready) begin
                    state_next = STATE_MEM_LOAD_WAIT;

                    addr_next = {alu_res[31:2] + 30'h1, 2'b0};

                    case (funct3)
                        3'b001, 3'b101: begin
                            // lh, lhu
                            case (alu_res[1:0])
                                2'b11: byte_enable_next = 4'b0001;
                                default: state_next = STATE_ERROR;
                            endcase
                        end
                        3'b010: begin
                            // lw
                            case (alu_res[1:0])
                                2'b01: byte_enable_next = 4'b0001;
                                2'b10: byte_enable_next = 4'b0011;
                                2'b11: byte_enable_next = 4'b0111;
                                default: state_next = STATE_ERROR;
                            endcase
                        end
                        default: state_next = STATE_ERROR;
                    endcase
                end
            end

            STATE_MEM_LOAD_WAIT: begin
                if (ready) begin
                    // Finish asserting reads
                    read_req_next = 0;

                    case (funct3)
                        3'b000, 3'b100: begin
                            // lb, lbu
                            if (read_buffer_count == 2'h1) begin
                                state_next = STATE_REG_WRITEBACK;
                            end
                        end
                        3'b001, 3'b101: begin
                            // lh, lhu
                            if (alu_res[1:0] == 2'b11) begin
                                if (read_buffer_count == 2'h2) begin
                                    state_next = STATE_REG_WRITEBACK;
                                end
                            end
                            else begin
                                if (read_buffer_count == 2'h1) begin
                                    state_next = STATE_REG_WRITEBACK;
                                end
                            end
                        end
                        3'b010: begin
                            // lw
                            if (alu_res[1:0] != 2'b00) begin
                                if (read_buffer_count == 2'h2) begin
                                    state_next = STATE_REG_WRITEBACK;
                                end
                            end
                            else begin
                                if (read_buffer_count == 2'h1) begin
                                    state_next = STATE_REG_WRITEBACK;
                                end
                            end
                        end
                        default: state_next = STATE_ERROR;
                    endcase
                end
            end

            STATE_MEM_STORE: begin
                state_next = STATE_REG_WRITEBACK;

                addr_next = {alu_res[31:2], 2'b0};
                write_req_next = 1;

                case (funct3)
                    3'b000: begin
                        // sb
                        byte_enable_next = 4'b0001 << alu_res[1:0];
                        case (alu_res[1:0])
                            2'b00: write_data_next = rs2_value;
                            2'b01: write_data_next = {16'b0, rs2_value[7:0], 8'b0};
                            2'b10: write_data_next = {8'b0, rs2_value[7:0], 16'b0};
                            2'b11: write_data_next = {rs2_value[7:0], 24'b0};
                        endcase
                    end
                    3'b001: begin
                        // sh
                        byte_enable_next = 4'b0011 << alu_res[1:0];
                        case (alu_res[1:0])
                            2'b00: write_data_next = rs2_value;
                            2'b01: write_data_next = {8'b0, rs2_value[15:0], 8'b0};
                            2'b10: write_data_next = {rs2_value[15:0], 16'b0};
                            2'b11: begin
                                write_data_next = {rs2_value[7:0], 24'b0};
                                state_next = STATE_MEM_STORE_2;
                            end
                        endcase
                    end
                    3'b010: begin
                        // sw
                        byte_enable_next = 4'b1111 << alu_res[1:0];
                        case (alu_res[1:0])
                            2'b00: write_data_next = rs2_value;
                            2'b01: begin
                                write_data_next = {rs2_value[23:0], 8'b0};
                                state_next = STATE_MEM_STORE_2;
                            end
                            2'b10: begin
                                write_data_next = {rs2_value[15:0], 16'b0};
                                state_next = STATE_MEM_STORE_2;
                            end
                            2'b11: begin
                                write_data_next = {rs2_value[7:0], 24'b0};
                                state_next = STATE_MEM_STORE_2;
                            end
                        endcase
                    end
                    default: state_next = STATE_ERROR;
                endcase
            end

            STATE_MEM_STORE_2: begin
                if (ready) begin
                    state_next = STATE_REG_WRITEBACK;

                    addr_next = {alu_res[31:2] + 30'h1, 2'b0};

                    case (funct3)
                        3'b001: begin
                            // sh
                            case (alu_res[1:0])
                                2'b11: begin
                                    write_data_next = {24'b0, rs2_value[15:8]};
                                    byte_enable_next = 4'b0001;
                                end
                                default: state_next = STATE_ERROR;
                            endcase
                        end
                        3'b010: begin
                            // sw
                            case (alu_res[1:0])
                                2'b01: begin
                                    write_data_next = {24'b0, rs2_value[31:24]};
                                    byte_enable_next = 4'b0001;
                                end
                                2'b10: begin
                                    write_data_next = {16'b0, rs2_value[31:16]};
                                    byte_enable_next = 4'b0011;
                                end
                                2'b11: begin
                                    write_data_next = {8'b0, rs2_value[31:8]};
                                    byte_enable_next = 4'b0111;
                                end
                                default: state_next = STATE_ERROR;
                            endcase
                        end
                        default: state_next = STATE_ERROR;
                    endcase
                end
            end

            STATE_REG_WRITEBACK: begin
                if (ready) begin
                    // Finish asserting writes, if any
                    write_req_next = 0;

                    state_next = STATE_INSTRUCTION_FETCH;
                    pc_next = pc + 32'h4;

                    case (opcode)
                        7'b0110111, 7'b0010111, 7'b0010011, 7'b0110011: begin
                            // lui, auipc, immediate computation, register computation
                            if (rd != 0) begin
                                regs_next[rd - 1] = alu_res;
                            end
                        end
                        7'b1101111: begin
                            // jal
                            if (rd != 0) begin
                                regs_next[rd - 1] = pc + 32'h4;
                            end

                            pc_next = alu_res;
                        end
                        7'b1100111: begin
                            // jalr
                            if (rd != 0) begin
                                regs_next[rd - 1] = pc + 32'h4;
                            end

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
                            if (rd != 0) begin
                                case (funct3)
                                    3'b000: begin
                                        // lb
                                        case (alu_res[1:0])
                                            2'b00: regs_next[rd - 1] = {{24{read_buffer_data[0][7]}}, read_buffer_data[0][7:0]};
                                            2'b01: regs_next[rd - 1] = {{24{read_buffer_data[0][15]}}, read_buffer_data[0][15:8]};
                                            2'b10: regs_next[rd - 1] = {{24{read_buffer_data[0][23]}}, read_buffer_data[0][23:16]};
                                            2'b11: regs_next[rd - 1] = {{24{read_buffer_data[0][31]}}, read_buffer_data[0][31:24]};
                                        endcase
                                    end
                                    3'b100: begin
                                        // lbu
                                        case (alu_res[1:0])
                                            2'b00: regs_next[rd - 1] = {24'b0, read_buffer_data[0][7:0]};
                                            2'b01: regs_next[rd - 1] = {24'b0, read_buffer_data[0][15:8]};
                                            2'b10: regs_next[rd - 1] = {24'b0, read_buffer_data[0][23:16]};
                                            2'b11: regs_next[rd - 1] = {24'b0, read_buffer_data[0][31:24]};
                                        endcase
                                    end
                                    3'b001: begin
                                        // lh
                                        case (alu_res[1:0])
                                            2'b00: regs_next[rd - 1] = {{16{read_buffer_data[0][15]}}, read_buffer_data[0][15:0]};
                                            2'b01: regs_next[rd - 1] = {{16{read_buffer_data[0][23]}}, read_buffer_data[0][23:8]};
                                            2'b10: regs_next[rd - 1] = {{16{read_buffer_data[0][31]}}, read_buffer_data[0][31:16]};
                                            2'b11: regs_next[rd - 1] = {{16{read_buffer_data[1][7]}}, read_buffer_data[1][7:0], read_buffer_data[0][31:24]};
                                        endcase
                                    end
                                    3'b101: begin
                                        // lhu
                                        case (alu_res[1:0])
                                            2'b00: regs_next[rd - 1] = {16'b0, read_buffer_data[0][15:0]};
                                            2'b01: regs_next[rd - 1] = {16'b0, read_buffer_data[0][23:8]};
                                            2'b10: regs_next[rd - 1] = {16'b0, read_buffer_data[0][31:16]};
                                            2'b11: regs_next[rd - 1] = {16'b0, read_buffer_data[1][7:0], read_buffer_data[0][31:24]};
                                        endcase
                                    end
                                    3'b010: begin
                                        // lw
                                        case (alu_res[1:0])
                                            2'b00: regs_next[rd - 1] = read_buffer_data[0];
                                            2'b01: regs_next[rd - 1] = {read_buffer_data[1][7:0], read_buffer_data[0][31:8]};
                                            2'b10: regs_next[rd - 1] = {read_buffer_data[1][15:0], read_buffer_data[0][31:16]};
                                            2'b11: regs_next[rd - 1] = {read_buffer_data[1][23:0], read_buffer_data[0][31:24]};
                                        endcase
                                    end
                                    default: state_next = STATE_ERROR;
                                endcase
                            end
                        end
                        7'b1110011: begin
                            // system instr's
                            case (funct3)
                                3'b001, 3'b010, 3'b011, 3'b101, 3'b110, 3'b111: begin
                                    // csrrw, csrrs, csrrc, csrrwi, csrrsi, csrrci
                                    if (rd != 0) begin
                                        case (csr)
                                            12'hc00, 12'hc01: regs_next[rd - 1] = cycle[31:0]; // cycle, time
                                            12'hc02: regs_next[rd - 1] = instret[31:0]; // instret
                                            12'hc80, 12'hc81: regs_next[rd - 1] = cycle[63:32]; // cycleh, timeh
                                            12'hc82: regs_next[rd - 1] = instret[63:32]; // instreth
                                            default: state_next = STATE_ERROR;
                                        endcase
                                    end
                                end
                                default: state_next = STATE_ERROR;
                            endcase
                        end
                    endcase

                    // Set up instruction fetch state
                    if (pc_next[1:0] != 2'b0) begin
                        // Misaligned PC
                        state_next = STATE_ERROR;
                    end
                    else begin
                        addr_next = pc_next;
                        byte_enable_next = 4'hf;
                        read_req_next = 1;

                        instret_next = instret + 64'h1;
                    end
                end
            end

            default: state_next = STATE_ERROR;
        endcase
    end

    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            addr <= 32'h0;
            write_data <= 32'h0;
            byte_enable <= 4'h0;
            write_req <= 0;
            read_req <= 0;

            pc <= 32'h10000000;

            regs <= '{default:0};

            cycle <= 64'h0;
            instret <= 64'h0;

            state <= STATE_INITIAL_INSTRUCTION_FETCH_SETUP;

            alu_op <= ADD;
            alu_lhs <= 32'h0;
            alu_rhs <= 32'h0;

            read_buffer_clear <= 0;

            instruction <= 32'h0;
        end
        else begin
            addr <= addr_next;
            write_data <= write_data_next;
            byte_enable <= byte_enable_next;
            write_req <= write_req_next;
            read_req <= read_req_next;

            pc <= pc_next;

            regs <= regs_next;

            cycle <= cycle_next;
            instret <= instret_next;

            state <= state_next;

            alu_op <= alu_op_next;
            alu_lhs <= alu_lhs_next;
            alu_rhs <= alu_rhs_next;

            read_buffer_clear <= read_buffer_clear_next;

            instruction <= instruction_next;
        end
    end

endmodule
