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


// ******************************************************************************************************************************** 
// This file instantiates the PLL.
// ******************************************************************************************************************************** 

`timescale 1 ps / 1 ps

(* altera_attribute = "-name IP_TOOL_NAME common; -name IP_TOOL_VERSION 17.1; -name FITTER_ADJUST_HC_SHORT_PATH_GUARDBAND 100; -name ALLOW_SYNCH_CTRL_USAGE OFF; -name AUTO_CLOCK_ENABLE_RECOGNITION OFF; -name AUTO_SHIFT_REGISTER_RECOGNITION OFF" *)

module ddr3_controller_pll0 (
	global_reset_n,
	pll_ref_clk,
	pll_locked,
	
	afi_clk,
	afi_half_clk,
	pll_mem_clk,
	pll_write_clk,
	pll_capture0_clk,
	pll_capture1_clk,
	
	scanclk,
	phasecounterselect,
	phasestep,
	phaseupdown,
	phasedone
);

// ******************************************************************************************************************************** 
// BEGIN PARAMETER SECTION
// All parameters default to "" will have their values passed in from higher level wrapper with the controller and driver. 
parameter DEVICE_FAMILY = "MAX 10";

// Clock settings
parameter REF_CLK_PERIOD_PS = 10000;

parameter PLL_AFI_CLK_DIV           = 2;
parameter PLL_MEM_CLK_DIV           = 1;
parameter PLL_WRITE_CLK_DIV         = 1;
parameter PLL_CAP0_CLK_DIV          = 1;
parameter PLL_CAP1_CLK_DIV          = 1;

parameter PLL_AFI_CLK_MULT          = 3;
parameter PLL_MEM_CLK_MULT          = 3;
parameter PLL_WRITE_CLK_MULT        = 3;
parameter PLL_CAP0_CLK_MULT         = 3;
parameter PLL_CAP1_CLK_MULT         = 3;

parameter PLL_AFI_CLK_PHASE_PS      = "0";
parameter PLL_MEM_CLK_PHASE_PS      = "0";
parameter PLL_WRITE_CLK_PHASE_PS    = "2500";
parameter PLL_CAP0_CLK_PHASE_PS     = "0";
parameter PLL_CAP1_CLK_PHASE_PS     = "2500";

// END PARAMETER SECTION
// ******************************************************************************************************************************** 

// ******************************************************************************************************************************** 
// BEGIN PORT SECTION

input   global_reset_n;		// Resets (active-low) the whole system (all PHY logic + PLL)
input	pll_ref_clk;		// PLL reference clock
output	pll_locked;    // When 0, PLL is out of lock

// When the PHY is selected to be a PLL/DLL MASTER, the PLL and DLL are instantied on this top level
output  afi_clk;		// See pll_memphy instantiation below for detailed description of each clock
output  afi_half_clk; 
output	pll_mem_clk;
output	pll_write_clk;
output	pll_capture0_clk;
output	pll_capture1_clk;

input	[2:0] phasecounterselect;
input	phasestep;
input	phaseupdown;
output	phasedone;
input	scanclk;

assign afi_half_clk = 1'b0;

// END PORT SECTION
// ******************************************************************************************************************************** 

    wire [4:0] sub_wire;
	wire  afi_clk         = sub_wire[4];
	wire  pll_mem_clk         = sub_wire[0];
	wire  pll_write_clk       = sub_wire[1];
	wire  pll_capture0_clk    = sub_wire[2];
	wire  pll_capture1_clk    = sub_wire[3];

	altpll	upll_memphy (
				.areset (~global_reset_n),
				.inclk ({1'h0, pll_ref_clk}),
				.phasecounterselect (phasecounterselect),
				.phasestep (phasestep),
				.scanclk (scanclk),
				.phaseupdown (phaseupdown),
				.clk (sub_wire),
				.locked (pll_locked),
				.phasedone (phasedone),
				.activeclock (),
				.clkbad (),
				.clkena ({6{1'b1}}),
				.clkloss (),
				.clkswitch (1'b0),
				.configupdate (1'b0),
				.enable0 (),
				.enable1 (),
				.extclk (),
				.extclkena ({4{1'b1}}),
				.fbin (1'b1),
				.fbmimicbidir (),
				.fbout (),
				.fref (),
				.icdrclk (),
				.pfdena (1'b1),
				.pllena (1'b1),
				.scanaclr (1'b0),
				.scanclkena (1'b1),
				.scandata (1'b0),
				.scandataout (),
				.scandone (),
				.scanread (1'b0),
				.scanwrite (1'b0),
				.sclkout0 (),
				.sclkout1 (),
				.vcooverrange (),
				.vcounderrange ());
	defparam
		upll_memphy.bandwidth_type = "AUTO",
		
		upll_memphy.clk4_divide_by = PLL_AFI_CLK_DIV,
		upll_memphy.clk4_duty_cycle = 50,
		upll_memphy.clk4_multiply_by = PLL_AFI_CLK_MULT,
		upll_memphy.clk4_phase_shift = PLL_AFI_CLK_PHASE_PS,
		
		upll_memphy.clk0_divide_by = PLL_MEM_CLK_DIV,
		upll_memphy.clk0_duty_cycle = 50,
		upll_memphy.clk0_multiply_by = PLL_MEM_CLK_MULT,
		upll_memphy.clk0_phase_shift = PLL_MEM_CLK_PHASE_PS,
		
		upll_memphy.clk1_divide_by = PLL_WRITE_CLK_DIV,
		upll_memphy.clk1_duty_cycle = 50,
		upll_memphy.clk1_multiply_by = PLL_WRITE_CLK_MULT,
		upll_memphy.clk1_phase_shift = PLL_WRITE_CLK_PHASE_PS,
		
		upll_memphy.clk2_divide_by = PLL_CAP0_CLK_DIV,
		upll_memphy.clk2_duty_cycle = 50,
		upll_memphy.clk2_multiply_by = PLL_CAP0_CLK_MULT,
		upll_memphy.clk2_phase_shift = PLL_CAP0_CLK_PHASE_PS,
		
		upll_memphy.clk3_divide_by = PLL_CAP1_CLK_DIV,
		upll_memphy.clk3_duty_cycle = 50,
		upll_memphy.clk3_multiply_by = PLL_CAP1_CLK_MULT,
		upll_memphy.clk3_phase_shift = PLL_CAP1_CLK_PHASE_PS,
		
		upll_memphy.compensate_clock = "CLK1",
		upll_memphy.inclk0_input_frequency = REF_CLK_PERIOD_PS,
		upll_memphy.intended_device_family = DEVICE_FAMILY,
		upll_memphy.lpm_type = "altpll",
		upll_memphy.operation_mode = "NORMAL",
		upll_memphy.pll_type = "AUTO",
		upll_memphy.port_activeclock = "PORT_UNUSED",
		upll_memphy.port_areset = "PORT_USED",
		upll_memphy.port_clkbad0 = "PORT_UNUSED",
		upll_memphy.port_clkbad1 = "PORT_UNUSED",
		upll_memphy.port_clkloss = "PORT_UNUSED",
		upll_memphy.port_clkswitch = "PORT_UNUSED",
		upll_memphy.port_configupdate = "PORT_UNUSED",
		upll_memphy.port_fbin = "PORT_UNUSED",
		upll_memphy.port_inclk0 = "PORT_USED",
		upll_memphy.port_inclk1 = "PORT_UNUSED",
		upll_memphy.port_locked = "PORT_USED",
		upll_memphy.port_pfdena = "PORT_UNUSED",
		upll_memphy.port_phasecounterselect = "PORT_USED",
		upll_memphy.port_phasedone = "PORT_USED",
		upll_memphy.port_phasestep = "PORT_USED",
		upll_memphy.port_phaseupdown = "PORT_USED",
		upll_memphy.port_pllena = "PORT_UNUSED",
		upll_memphy.port_scanaclr = "PORT_UNUSED",
		upll_memphy.port_scanclk = "PORT_USED",
		upll_memphy.port_scanclkena = "PORT_UNUSED",
		upll_memphy.port_scandata = "PORT_UNUSED",
		upll_memphy.port_scandataout = "PORT_UNUSED",
		upll_memphy.port_scandone = "PORT_UNUSED",
		upll_memphy.port_scanread = "PORT_UNUSED",
		upll_memphy.port_scanwrite = "PORT_UNUSED",
		upll_memphy.port_clk0 = "PORT_USED",
		upll_memphy.port_clk1 = "PORT_USED",
		upll_memphy.port_clk2 = "PORT_USED",
		upll_memphy.port_clk3 = "PORT_USED",
		upll_memphy.port_clk4 = "PORT_USED",
		upll_memphy.port_clk5 = "PORT_UNUSED",
		upll_memphy.port_clkena0 = "PORT_UNUSED",
		upll_memphy.port_clkena1 = "PORT_UNUSED",
		upll_memphy.port_clkena2 = "PORT_UNUSED",
		upll_memphy.port_clkena3 = "PORT_UNUSED",
		upll_memphy.port_clkena4 = "PORT_UNUSED",
		upll_memphy.port_clkena5 = "PORT_UNUSED",
		upll_memphy.port_extclk0 = "PORT_UNUSED",
		upll_memphy.port_extclk1 = "PORT_UNUSED",
		upll_memphy.port_extclk2 = "PORT_UNUSED",
		upll_memphy.port_extclk3 = "PORT_UNUSED",
		upll_memphy.self_reset_on_loss_lock = "OFF",
		upll_memphy.vco_frequency_control = "MANUAL_PHASE",
		upll_memphy.vco_phase_shift_step = 104,
		upll_memphy.width_clock = 5,
		upll_memphy.width_phasecounterselect = 3;

endmodule

