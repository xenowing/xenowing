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

    localparam STATE_INSTRUCTION_FETCH = 3'h0;
    localparam STATE_ERROR = 3'h1;
    localparam STATE_INSTRUCTION_FETCH_WAIT = 3'h2;
    localparam STATE_INSTRUCTION_DECODE = 3'h3;
    localparam STATE_STORE_HAXX = 3'h4;
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
    logic [31:0] store_offset;
    logic [31:0] jump_offset;
    logic [31:0] i_immediate;
    logic [31:0] u_immediate;
    assign opcode = instruction[6:0];
    assign rd = instruction[11:7];
    assign rs1 = instruction[19:15];
    assign rs2 = instruction[24:20];
    assign funct3 = instruction[14:12];
    assign store_offset = {{20{instruction[31]}}, {instruction[31:25], instruction[11:7]}};
    assign jump_offset = {{11{instruction[31]}}, {instruction[31], instruction[19:12], instruction[20], instruction[30:21]}, 1'b0};
    assign i_immediate = {{20{instruction[31]}}, instruction[31:20]};
    assign u_immediate = {instruction[31:12], 12'h0};

    alu alu0(
        .op(alu_op),
        .lhs(alu_lhs),
        .rhs(alu_rhs),
        .res(alu_res));

    always_comb begin
        addr_next = addr;
        write_data_next = write_data;
        byte_enable_next = byte_enable;
        write_req_next = write_req;
        read_req_next = read_req;

        pc_next = pc;

        regs_next = regs;

        state_next = state;

        alu_op_next = alu_op;
        alu_lhs_next = alu_lhs;
        alu_rhs_next = alu_rhs;

        instruction_next = instruction;

        case (state)
            STATE_INSTRUCTION_FETCH: begin
                // Fetch word at PC
                addr_next = pc;
                byte_enable_next = 4'hf;
                read_req_next = 1;

                state_next = STATE_INSTRUCTION_FETCH_WAIT;
            end

            STATE_ERROR: begin
                // TODO
            end

            STATE_INSTRUCTION_FETCH_WAIT: begin
                if (ready) begin
                    read_req_next = 0;

                    if (read_data_valid) begin
                        instruction_next = read_data;

                        state_next = STATE_INSTRUCTION_DECODE;
                    end
                end
            end

            STATE_INSTRUCTION_DECODE: begin
                case (opcode)
                    7'b0010011: begin
                        case (funct3)
                            3'b000: begin
                                // addi
                                if (rd != 0) begin
                                    regs_next[rd - 1] = (rs1 > 0 ? regs[rs1 - 1] : 32'h0) + i_immediate;
                                end

                                pc_next = pc + 32'h4;

                                state_next = STATE_INSTRUCTION_FETCH;
                            end
                            default: state_next = STATE_ERROR;
                        endcase
                    end
                    7'b0100011: begin
                        case (funct3)
                            3'b010: begin
                                // sw
                                addr_next = (rs1 > 0 ? regs[rs1 - 1] : 32'h0) + store_offset;
                                write_data_next = rs2 > 0 ? regs[rs2 - 1] : 32'h0;
                                byte_enable_next = 4'hf;
                                write_req_next = 1;

                                state_next = STATE_STORE_HAXX;
                            end
                            default: state_next = STATE_ERROR;
                        endcase
                    end
                    7'b1101111: begin
                        // jal
                        if (rd != 0) begin
                            regs_next[rd - 1] = pc + 4;
                        end

                        pc_next = pc + jump_offset;

                        state_next = STATE_INSTRUCTION_FETCH;
                    end
                    7'b0110111: begin
                        // lui
                        if (rd != 0) begin
                            regs_next[rd - 1] = u_immediate;
                        end

                        pc_next = pc + 32'h4;

                        state_next = STATE_INSTRUCTION_FETCH;
                    end
                    default: state_next = STATE_ERROR;
                endcase
            end

            STATE_STORE_HAXX: begin
                if (ready) begin
                    write_req_next = 0;

                    pc_next = pc + 32'h4;

                    state_next = STATE_INSTRUCTION_FETCH;
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

            state <= STATE_INSTRUCTION_FETCH;

            alu_op <= ADD;
            alu_lhs <= 32'h0;
            alu_rhs <= 32'h0;

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

            state <= state_next;

            alu_op <= alu_op_next;
            alu_lhs <= alu_lhs_next;
            alu_rhs <= alu_rhs_next;

            instruction <= instruction_next;
        end
    end

endmodule
