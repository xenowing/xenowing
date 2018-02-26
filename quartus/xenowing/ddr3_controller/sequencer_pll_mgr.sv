// (C) 2001-2017 Intel Corporation. All rights reserved.
// Your use of Intel Corporation's design tools, logic functions and other 
// software and tools, and its AMPP partner logic functions, and any output 
// files from any of the foregoing (including device programming or simulation 
// files), and any associated documentation or information are expressly subject 
// to the terms and conditions of the Intel Program License Subscription 
// Agreement, Intel FPGA IP License Agreement, or other applicable 
// license agreement, including, without limitation, that your use is for the 
// sole purpose of programming logic devices manufactured by Intel and sold by 
// Intel or its authorized distributors.  Please refer to the applicable 
// agreement for further details.


// ******
// pll_mgr
// ******
//
// PLL Manager
//
// General Description
// -------------------
//
// This component allows a way for a master on the Avalon bus to
// control PLL output clock phase during calibration

//

`timescale 1 ps / 1 ps

module sequencer_pll_mgr (
	// Avalon Interface
	
	avl_clk,
	avl_reset_n,
	avl_address,
	avl_write,
	avl_writedata,
	avl_read,
	avl_readdata,
	avl_waitrequest,

	// PLL control

	phasecounterselect,
	phasestep,
	phaseupdown,
	scanclk,
	phasedone,
	
	afi_init_req,
	afi_cal_req
);

	parameter AVL_DATA_WIDTH = 32;
	parameter AVL_ADDR_WIDTH = 1;
	
	localparam PHASE_COUNTER_WIDTH = 16;
	localparam PLL_COUNTER_BIT_WIDTH = 3;
	localparam PLL_COUNTER = 2;

	input avl_clk;
	input avl_reset_n;
	input [AVL_ADDR_WIDTH - 1:0] avl_address;
	input avl_write;
	input [AVL_DATA_WIDTH - 1:0] avl_writedata;
	input avl_read;
	output [AVL_DATA_WIDTH - 1:0] avl_readdata;
	output avl_waitrequest;

	output [PLL_COUNTER_BIT_WIDTH - 1:0] phasecounterselect;
	output phasestep;
	output phaseupdown;
	output scanclk;
	input phasedone;
	
	input afi_init_req;
	input afi_cal_req;
	
	logic scanclk;
	logic [PHASE_COUNTER_WIDTH - 1:0] mem_array [PLL_COUNTER - 1:0];
	logic [AVL_DATA_WIDTH - 1:0] avl_readdata;
	logic int_waitrequest;
	
	logic [AVL_ADDR_WIDTH - 1:0] int_address;
	logic [AVL_DATA_WIDTH - 1:0] int_writedata;
	
	logic [PLL_COUNTER_BIT_WIDTH - 1:0] phasecounterselect;
	logic phasestep;
	logic phaseupdown;
	
	typedef enum int unsigned {
		IDLE,
		WRITE,
		PLLWAIT,
		PLLWAIT2,
		PLLWAIT3,
		READ,
		READWAIT
	} PLL_MGR_STATE;

	PLL_MGR_STATE state;
	
	assign scanclk = avl_clk;
	assign avl_waitrequest = ((state == IDLE) && ((avl_read == 1) || (avl_write == 1))) ? 1'b1 : int_waitrequest;
	
	// State machine
		
	always_ff @ (posedge avl_clk or negedge avl_reset_n)
	    begin
	        if (avl_reset_n == 0)
	            begin
	                state <= IDLE;
	                avl_readdata <= 0;
	                int_waitrequest <= 0;
	                for (int i=0; i < PLL_COUNTER; i++)
	                    mem_array[i] <= 0;
	                phasecounterselect <= 0;
	                phasestep <= 0;
	                phaseupdown <= 0;
	            end
	        else
	            begin
	                case(state)
	                    IDLE :
	                        begin
	                            int_waitrequest <= 0;
	                            if (avl_write)
	                                begin
	                                    state <= WRITE;
	                                    int_waitrequest <= 1;
	                                end
	                            else if (avl_read)
	                                begin
	                                    state <= READ;
	                                    int_waitrequest <= 1;
	                                end
	                        end
	                    WRITE :
	                        begin
	                            if (int_writedata[0]) 
	                                mem_array [int_address]    <=    mem_array [int_address] + 1'b1;
	                            else
	                                mem_array [int_address]    <=    mem_array [int_address] - 1'b1;
	                            
	                            phaseupdown <= int_writedata[0];
	                            phasecounterselect <= int_address[2:0] + 3'd4;
	                            phasestep <= 1;
	                            state <= PLLWAIT;
	                        end
	                    PLLWAIT :
	                        state <= PLLWAIT2;
	                    PLLWAIT2 :
	                        begin
	                            state <= PLLWAIT3;
	                            int_waitrequest <= 0;
	                        end
	                    PLLWAIT3 :
	                        begin
	                            phasestep <= 0;
	                            state <= IDLE;
	                        end
	                    READ :
	                        begin
	                            if (int_address < PLL_COUNTER)
	                                avl_readdata <= mem_array [int_address];
	                            else
	                                avl_readdata <= afi_init_req | afi_cal_req;
	                            int_waitrequest <= 0;
	                            state <= READWAIT;
	                        end
	                    READWAIT :
	                        begin
	                            state <= IDLE;
	                        end
	                endcase
	            end
	    end
	
	always_ff @ (posedge avl_clk or negedge avl_reset_n)
	    begin
	        if (avl_reset_n == 0)
	            begin
	                int_address <= 0;
	                int_writedata <= 0;
	            end
	        else
	            begin
	                int_address <= avl_address;
	                int_writedata <= avl_writedata;
	            end
	    end
	
endmodule
