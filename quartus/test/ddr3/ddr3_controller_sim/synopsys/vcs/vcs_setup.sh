
# (C) 2001-2019 Altera Corporation. All rights reserved.
# Your use of Altera Corporation's design tools, logic functions and 
# other software and tools, and its AMPP partner logic functions, and 
# any output files any of the foregoing (including device programming 
# or simulation files), and any associated documentation or information 
# are expressly subject to the terms and conditions of the Altera 
# Program License Subscription Agreement, Altera MegaCore Function 
# License Agreement, or other applicable license agreement, including, 
# without limitation, that your use is for the sole purpose of 
# programming logic devices manufactured by Altera and sold by Altera 
# or its authorized distributors. Please refer to the applicable 
# agreement for further details.

# ACDS 18.1 646 win32 2019.09.02.20:36:13

# ----------------------------------------
# vcs - auto-generated simulation script

# ----------------------------------------
# This script provides commands to simulate the following IP detected in
# your Quartus project:
#     ddr3_controller
# 
# Altera recommends that you source this Quartus-generated IP simulation
# script from your own customized top-level script, and avoid editing this
# generated script.
# 
# To write a top-level shell script that compiles Altera simulation libraries
# and the Quartus-generated IP in your project, along with your design and
# testbench files, follow the guidelines below.
# 
# 1) Copy the shell script text from the TOP-LEVEL TEMPLATE section
# below into a new file, e.g. named "vcs_sim.sh".
# 
# 2) Copy the text from the DESIGN FILE LIST & OPTIONS TEMPLATE section into
# a separate file, e.g. named "filelist.f".
# 
# ----------------------------------------
# # TOP-LEVEL TEMPLATE - BEGIN
# #
# # TOP_LEVEL_NAME is used in the Quartus-generated IP simulation script to
# # set the top-level simulation or testbench module/entity name.
# #
# # QSYS_SIMDIR is used in the Quartus-generated IP simulation script to
# # construct paths to the files required to simulate the IP in your Quartus
# # project. By default, the IP script assumes that you are launching the
# # simulator from the IP script location. If launching from another
# # location, set QSYS_SIMDIR to the output directory you specified when you
# # generated the IP script, relative to the directory from which you launch
# # the simulator.
# #
# # Source the Quartus-generated IP simulation script and do the following:
# # - Compile the Quartus EDA simulation library and IP simulation files.
# # - Specify TOP_LEVEL_NAME and QSYS_SIMDIR.
# # - Compile the design and top-level simulation module/entity using
# #   information specified in "filelist.f".
# # - Override the default USER_DEFINED_SIM_OPTIONS. For example, to run
# #   until $finish(), set to an empty string: USER_DEFINED_SIM_OPTIONS="".
# # - Run the simulation.
# #
# source <script generation output directory>/synopsys/vcs/vcs_setup.sh \
# TOP_LEVEL_NAME=<simulation top> \
# QSYS_SIMDIR=<script generation output directory> \
# USER_DEFINED_ELAB_OPTIONS="\"-f filelist.f\"" \
# USER_DEFINED_SIM_OPTIONS=<simulation options for your design>
# #
# # TOP-LEVEL TEMPLATE - END
# ----------------------------------------
# 
# ----------------------------------------
# # DESIGN FILE LIST & OPTIONS TEMPLATE - BEGIN
# #
# # Compile all design files and testbench files, including the top level.
# # (These are all the files required for simulation other than the files
# # compiled by the Quartus-generated IP simulation script)
# #
# +systemverilogext+.sv
# <design and testbench files, compile-time options, elaboration options>
# #
# # DESIGN FILE LIST & OPTIONS TEMPLATE - END
# ----------------------------------------
# 
# IP SIMULATION SCRIPT
# ----------------------------------------
# If ddr3_controller is one of several IP cores in your
# Quartus project, you can generate a simulation script
# suitable for inclusion in your top-level simulation
# script by running the following command line:
# 
# ip-setup-simulation --quartus-project=<quartus project>
# 
# ip-setup-simulation will discover the Altera IP
# within the Quartus project, and generate a unified
# script which supports all the Altera IP within the design.
# ----------------------------------------
# ACDS 18.1 646 win32 2019.09.02.20:36:13
# ----------------------------------------
# initialize variables
TOP_LEVEL_NAME="ddr3_controller"
QSYS_SIMDIR="./../../"
QUARTUS_INSTALL_DIR="C:/intelfpga_lite/18.1/quartus/"
SKIP_FILE_COPY=0
SKIP_SIM=0
USER_DEFINED_ELAB_OPTIONS=""
USER_DEFINED_SIM_OPTIONS="+vcs+finish+100"
# ----------------------------------------
# overwrite variables - DO NOT MODIFY!
# This block evaluates each command line argument, typically used for 
# overwriting variables. An example usage:
#   sh <simulator>_setup.sh SKIP_SIM=1
for expression in "$@"; do
  eval $expression
  if [ $? -ne 0 ]; then
    echo "Error: This command line argument, \"$expression\", is/has an invalid expression." >&2
    exit $?
  fi
done

# ----------------------------------------
# initialize simulation properties - DO NOT MODIFY!
ELAB_OPTIONS=""
SIM_OPTIONS=""
if [[ `vcs -platform` != *"amd64"* ]]; then
  :
else
  :
fi

# ----------------------------------------
# copy RAM/ROM files to simulation directory
if [ $SKIP_FILE_COPY -eq 0 ]; then
  cp -f $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_AC_ROM.hex ./
  cp -f $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_inst_ROM.hex ./
fi

vcs -lca -timescale=1ps/1ps -sverilog +verilog2001ext+.v -ntb_opts dtm $ELAB_OPTIONS $USER_DEFINED_ELAB_OPTIONS \
  -v $QUARTUS_INSTALL_DIR/eda/sim_lib/altera_primitives.v \
  -v $QUARTUS_INSTALL_DIR/eda/sim_lib/220model.v \
  -v $QUARTUS_INSTALL_DIR/eda/sim_lib/sgate.v \
  -v $QUARTUS_INSTALL_DIR/eda/sim_lib/altera_mf.v \
  $QUARTUS_INSTALL_DIR/eda/sim_lib/altera_lnsim.sv \
  -v $QUARTUS_INSTALL_DIR/eda/sim_lib/fiftyfivenm_atoms.v \
  -v $QUARTUS_INSTALL_DIR/eda/sim_lib/synopsys/fiftyfivenm_atoms_ncrypt.v \
  $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_mm_st_converter.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_addr_cmd.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_addr_cmd_wrap.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ddr2_odt_gen.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ddr3_odt_gen.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_lpddr2_addr_cmd.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_odt_gen.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_rdwr_data_tmg.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_arbiter.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_burst_gen.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_cmd_gen.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_csr.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_buffer.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_buffer_manager.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_burst_tracking.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_dataid_manager.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_fifo.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_list.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_rdata_path.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_wdata_path.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ecc_decoder.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ecc_decoder_32_syn.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ecc_decoder_64_syn.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ecc_encoder.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ecc_encoder_32_syn.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ecc_encoder_64_syn.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_ecc_encoder_decoder_wrapper.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_axi_st_converter.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_input_if.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_rank_timer.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_sideband.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_tbp.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_timing_param.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_controller.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_ddrx_controller_st_top.v \
  \"+incdir+$QSYS_SIMDIR/ddr3_controller/\" $QSYS_SIMDIR/ddr3_controller/alt_mem_if_nextgen_ddr3_controller_core.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_c0.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0.v \
  $QSYS_SIMDIR/ddr3_controller/altera_avalon_sc_fifo.v \
  $QSYS_SIMDIR/ddr3_controller/altera_mem_if_sequencer_rst.sv \
  $QSYS_SIMDIR/ddr3_controller/altera_merlin_arbitrator.sv \
  $QSYS_SIMDIR/ddr3_controller/altera_merlin_burst_uncompressor.sv \
  $QSYS_SIMDIR/ddr3_controller/altera_merlin_master_agent.sv \
  $QSYS_SIMDIR/ddr3_controller/altera_merlin_master_translator.sv \
  $QSYS_SIMDIR/ddr3_controller/altera_merlin_slave_agent.sv \
  $QSYS_SIMDIR/ddr3_controller/altera_merlin_slave_translator.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0_avalon_st_adapter.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0_avalon_st_adapter_error_adapter_0.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0_cmd_demux.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0_cmd_mux.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0_router.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0_router_001.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0_rsp_demux.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_s0_mm_interconnect_0_rsp_mux.sv \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_ac_ROM_reg.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_bitcheck.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_core.sv \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_datamux.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_data_broadcast.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_data_decoder.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_ddr3.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_di_buffer.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_di_buffer_wrap.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_dm_decoder.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_generic.sv \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_inst_ROM_reg.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_jumplogic.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_lfsr12.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_lfsr36.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_lfsr72.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_pattern_fifo.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_ram.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_ram_csr.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_read_datapath.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_write_decoder.v \
  $QSYS_SIMDIR/ddr3_controller/sequencer_m10.sv \
  $QSYS_SIMDIR/ddr3_controller/sequencer_phy_mgr.sv \
  $QSYS_SIMDIR/ddr3_controller/sequencer_pll_mgr.sv \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_m10_ac_ROM.v \
  $QSYS_SIMDIR/ddr3_controller/rw_manager_m10_inst_ROM.v \
  $QSYS_SIMDIR/ddr3_controller/afi_mux_ddr3_ddrx.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_clock_pair_generator.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_read_valid_selector.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_addr_cmd_datapath.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_reset_m10.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_memphy_m10.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_dqdqs_pads_m10.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_reset_sync.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_fr_cycle_shifter.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_read_datapath_m10.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_write_datapath_m10.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_simple_ddio_out_m10.sv \
  $QSYS_SIMDIR/ddr3_controller/max10emif_dcfifo.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_iss_probe.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_addr_cmd_pads_m10.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0_flop_mem.v \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_p0.sv \
  $QSYS_SIMDIR/ddr3_controller/altera_gpio_lite.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_pll0.sv \
  $QSYS_SIMDIR/ddr3_controller/ddr3_controller_0002.v \
  $QSYS_SIMDIR/ddr3_controller.v \
  -top $TOP_LEVEL_NAME
# ----------------------------------------
# simulate
if [ $SKIP_SIM -eq 0 ]; then
  ./simv $SIM_OPTIONS $USER_DEFINED_SIM_OPTIONS
fi
