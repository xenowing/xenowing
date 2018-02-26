# (C) 2001-2017 Intel Corporation. All rights reserved.
# Your use of Intel Corporation's design tools, logic functions and other 
# software and tools, and its AMPP partner logic functions, and any output 
# files from any of the foregoing (including device programming or simulation 
# files), and any associated documentation or information are expressly subject 
# to the terms and conditions of the Intel Program License Subscription 
# Agreement, Intel FPGA IP License Agreement, or other applicable 
# license agreement, including, without limitation, that your use is for the 
# sole purpose of programming logic devices manufactured by Intel and sold by 
# Intel or its authorized distributors.  Please refer to the applicable 
# agreement for further details.


#####################################################################
#
# THIS IS AN AUTO-GENERATED FILE!
# -------------------------------
# If you modify this files, all your changes will be lost if you
# regenerate the core!
#
# FILE DESCRIPTION
# ----------------
# This file contains the timing constraints for the UniPHY memory
# interface.
#    * The timing parameters used by this file are assigned
#      in the ddr3_controller_p0_timing.tcl script.
#    * The helper routines are defined in ddr3_controller_p0_pin_map.tcl
#
# NOTE
# ----

set script_dir [file dirname [info script]]

source "$script_dir/ddr3_controller_p0_parameters.tcl"
source "$script_dir/ddr3_controller_p0_timing.tcl"
source "$script_dir/ddr3_controller_p0_pin_map.tcl"

load_package ddr_timing_model

set synthesis_flow 0
set sta_flow 0
set fit_flow 0
if { $::TimeQuestInfo(nameofexecutable) == "quartus_map" } {
	set synthesis_flow 1
} elseif { $::TimeQuestInfo(nameofexecutable) == "quartus_sta" } {
	set sta_flow 1
} elseif { $::TimeQuestInfo(nameofexecutable) == "quartus_fit" } {
	set fit_flow 1
}

set is_es 0
if { [string match -nocase "*es" $::TimeQuestInfo(part)] } {
	set is_es 1
}

####################
#                  #
# GENERAL SETTINGS #
#                  #
####################

# This is a global setting and will apply to the whole design.
# This setting is required for the memory interface to be
# properly constrained.
derive_clock_uncertainty

# Debug switch. Change to 1 to get more run-time debug information
set debug 0

# All timing requirements will be represented in nanoseconds with up to 3 decimal places of precision
set_time_format -unit ns -decimal_places 3

# Determine if entity names are on
set entity_names_on [ ddr3_controller_p0_are_entity_names_on ]

##################
#                #
# QUERIED TIMING #
#                #
##################

set io_standard "$::GLOBAL_ddr3_controller_p0_io_standard CLASS I"


set outputDQSpathjitter 0.060
set outputDQSpathjitter_setup_prop 0.25
set tJITper 0.253 

##################
#                #
# DERIVED TIMING #
#                #
##################

# These parameters are used to make constraints more readeable

# Half of memory clock cycle
set half_period [ ddr3_controller_p0_round_3dp [ expr $t(CK) / 2.0 ] ]

# Half of reference clock
set ref_half_period [ ddr3_controller_p0_round_3dp [ expr $t(refCK) / 2.0 ] ]

# AFI cycle
set tCK_AFI [ expr $t(CK) * 2.0 ]

# Minimum delay on data output pins
set t(wru_output_min_delay_external) [expr $t(DH) + $board(intra_DQS_group_skew)/2 + $ISI(DQ)/2 + $ISI(DQS)/2 - $board(DQ_DQS_skew)]
set t(wru_output_min_delay_internal) [expr $outputDQSpathjitter*(1.0-$outputDQSpathjitter_setup_prop) + $SSN(rel_pullin_o)]
set data_output_min_delay [ ddr3_controller_p0_round_3dp [ expr - $t(wru_output_min_delay_external) - $t(wru_output_min_delay_internal)]]

# Maximum delay on data output pins
set t(wru_output_max_delay_external) [expr $t(DS) + $board(intra_DQS_group_skew)/2 + $ISI(DQ)/2 + $ISI(DQS)/2 + $board(DQ_DQS_skew)]
set t(wru_output_max_delay_internal) [expr $outputDQSpathjitter*$outputDQSpathjitter_setup_prop + $SSN(rel_pushout_o)]
set data_output_max_delay [ ddr3_controller_p0_round_3dp [ expr $t(wru_output_max_delay_external) + $t(wru_output_max_delay_internal)]]

# Minimum delay on address and command paths
set t(ac_output_min_delay_external) [expr $t(IH) + $board(intra_addr_ctrl_skew) + $ISI(addresscmd_hold) - $board(addresscmd_CK_skew)]
set t(ac_output_min_delay_internal) [expr $outputDQSpathjitter*(1.0-$outputDQSpathjitter_setup_prop) + $SSN(rel_pullin_o)]
set ac_min_delay [ ddr3_controller_p0_round_3dp [ expr - $t(ac_output_min_delay_external) - $t(ac_output_min_delay_internal)]]

# Maximum delay on address and command paths
set t(ac_output_max_delay_external) [expr $t(IS) + $board(intra_addr_ctrl_skew) + $ISI(addresscmd_setup) + $board(addresscmd_CK_skew)]
set t(ac_output_max_delay_internal) [expr $outputDQSpathjitter*$outputDQSpathjitter_setup_prop + $SSN(rel_pushout_o)]
set ac_max_delay [ ddr3_controller_p0_round_3dp [ expr $t(ac_output_max_delay_external) + $t(ac_output_max_delay_internal)]]

if { $debug } {
	post_message -type info "SDC: Computed Parameters:"
	post_message -type info "SDC: --------------------"
	post_message -type info "SDC: half_period: $half_period"
	post_message -type info "SDC: data_output_min_delay: $data_output_min_delay"
	post_message -type info "SDC: data_output_max_delay: $data_output_max_delay"
	post_message -type info "SDC: data_input_min_delay: $data_input_min_delay"
	post_message -type info "SDC: data_input_max_delay: $data_input_max_delay"
	post_message -type info "SDC: ac_min_delay: $ac_min_delay"
	post_message -type info "SDC: ac_max_delay: $ac_max_delay"
	post_message -type info "SDC: Using Timing Models: Micro"
}

# This is the main call to the netlist traversal routines
# that will automatically find all pins and registers required
# to apply timing constraints.
# During the fitter, the routines will be called only once
# and cached data will be used in all subsequent calls.
if { ! [ info exists ddr3_controller_p0_sdc_cache ] } {
	set ddr3_controller_p0_sdc_cache 1
	ddr3_controller_p0_initialize_ddr_db ddr3_controller_p0_ddr_db
} else {
	if { $debug } {
		post_message -type info "SDC: reusing cached DDR DB"
	}
}

# If multiple instances of this core are present in the
# design they will all be constrained through the
# following loop
set instances [ array names ddr3_controller_p0_ddr_db ]
foreach { inst } $instances {
	if { [ info exists pins ] } {
		# Clean-up stale content
		unset pins
	}
	array set pins $ddr3_controller_p0_ddr_db($inst)

	set prefix $inst
	if { $entity_names_on } {
		set prefix [ string map "| |*:" $inst ]
		set prefix "*:$prefix"
	}

	#####################################################
	#                                                   #
	# Transfer the pin names to more readable variables #
	#                                                   #
	#####################################################

	set dqs_pins $pins(dqs_pins)
	set dqsn_pins $pins(dqsn_pins)
	set q_groups [ list ]
	foreach { q_group } $pins(q_groups) {
		set q_group $q_group
		lappend q_groups $q_group
	}
	set all_dq_pins [ join [ join $q_groups ] ]

	set ck_pins $pins(ck_pins)
	set ckn_pins $pins(ckn_pins)
	set add_pins $pins(add_pins)
	set ba_pins $pins(ba_pins)
	set cmd_pins $pins(cmd_pins)
	set reset_pins $pins(reset_pins)
	set ac_pins [ concat $add_pins $ba_pins $cmd_pins ]
	set dm_pins $pins(dm_pins)
	set all_dq_dm_pins [ concat $all_dq_pins $dm_pins ]

	set pll_ref_clock $pins(pll_ref_clock)
	set pll_afi_clock $pins(pll_afi_clock)
	set pll_dq_write_clock $pins(pll_dq_write_clock)
	set pll_write_clock $pins(pll_write_clock)

	set dqs_in_clocks $pins(dqs_in_clocks)
	set dqs_out_clocks $pins(dqs_out_clocks)
	set dqsn_out_clocks $pins(dqsn_out_clocks)

	set afi_reset_reg $pins(afi_reset_reg)
	set seq_reset_reg $pins(seq_reset_reg)
	set sync_reg $pins(sync_reg)
	set read_capture_ddio $pins(read_capture_ddio)
	set fifo_wraddress_reg $pins(fifo_wraddress_reg)
	set fifo_rdaddress_reg $pins(fifo_rdaddress_reg)
	set fifo_wrload_reg $pins(fifo_wrload_reg)
	set fifo_rdload_reg $pins(fifo_rdload_reg)
	set fifo_wrdata_reg $pins(fifo_wrdata_reg)
	set fifo_rddata_reg $pins(fifo_rddata_reg)

	# ----------------------- #
	# -                     - #
	# --- REFERENCE CLOCK --- #
	# -                     - #
	# ----------------------- #

	# This is the reference clock used by the PLL to derive any other clock in the core
	if { [get_collection_size [get_clocks -nowarn $pll_ref_clock]] > 0 } { remove_clock $pll_ref_clock }
	create_clock -period $t(refCK) -waveform [ list 0 $ref_half_period ] $pll_ref_clock

	# ------------------ #
	# -                - #
	# --- PLL CLOCKS --- #
	# -                - #
	# ------------------ #
	
	# AFI clock
	set local_pll_afi_clk [ ddr3_controller_p0_get_or_add_clock_vseries \
		-target $pll_afi_clock \
		-suffix "afi_clk" \
		-source $pll_ref_clock \
		-multiply_by $::GLOBAL_ddr3_controller_p0_pll_mult(PLL_AFI_CLK) \
		-divide_by $::GLOBAL_ddr3_controller_p0_pll_div(PLL_AFI_CLK) \
		-phase $::GLOBAL_ddr3_controller_p0_pll_phase(PLL_AFI_CLK) ]
	
	# DQ write clock
	set local_pll_dq_write_clk [ ddr3_controller_p0_get_or_add_clock_vseries \
		-target $pll_dq_write_clock \
		-suffix "dq_write_clk" \
		-source $pll_ref_clock \
		-multiply_by $::GLOBAL_ddr3_controller_p0_pll_mult(PLL_WRITE_CLK) \
		-divide_by $::GLOBAL_ddr3_controller_p0_pll_div(PLL_WRITE_CLK) \
		-phase $::GLOBAL_ddr3_controller_p0_pll_phase(PLL_WRITE_CLK) ]

    	# DQS write clock
  	set local_pll_write_clk [ ddr3_controller_p0_get_or_add_clock_vseries \
  		-target $pll_write_clock \
  		-suffix "write_clk" \
  		-source $pll_ref_clock \
  		-multiply_by $::GLOBAL_ddr3_controller_p0_pll_mult(PLL_MEM_CLK) \
  		-divide_by $::GLOBAL_ddr3_controller_p0_pll_div(PLL_MEM_CLK) \
  		-phase $::GLOBAL_ddr3_controller_p0_pll_phase(PLL_MEM_CLK) ]
  
 	# -------------------- #
	# -                  - #
	# --- SYSTEM CLOCK --- #
	# -                  - #
	# -------------------- #

	# This is the CK clock
	foreach { ck_pin } $ck_pins {
		create_generated_clock -multiply_by 1 -source $pll_write_clock -master_clock "$local_pll_write_clk" $ck_pin -name $ck_pin
		set_clock_uncertainty -to [get_clocks $ck_pin] 0.025
	}

	# This is the CK#clock
	foreach { ckn_pin } $ckn_pins {
		create_generated_clock -multiply_by 1 -invert -source $pll_write_clock -master_clock "$local_pll_write_clk" $ckn_pin -name $ckn_pin
		set_clock_uncertainty -to [get_clocks $ckn_pin] 0.025
	}

        # ------------------- #
        # -                 - #
        # --- READ CLOCKS --- #
        # -                 - #
        # ------------------- #
        
        foreach dqs_in_clock_struct $dqs_in_clocks {
        	array set dqs_in_clock $dqs_in_clock_struct
        	# This is the DQS clock for Read Capture analysis (micro model)
        	create_clock -period $t(CK) -waveform [ list 0 $half_period ] $dqs_in_clock(dqs_pin) -name $dqs_in_clock(dqs_pin)_IN -add
        
        	# Clock Uncertainty is accounted for by the ...pathjitter parameters
        	set_clock_uncertainty -from [ get_clocks $dqs_in_clock(dqs_pin)_IN ] 0
        }
        
        	set resync_clock ${inst}|pll0|upll_memphy|auto_generated|pll1|clk\[2\]
          	
                set local_resync_clk [ ddr3_controller_p0_get_or_add_clock_vseries \
          		-target $resync_clock \
          		-suffix "resync_clk" \
          		-source $pll_ref_clock \
          		-multiply_by $::GLOBAL_ddr3_controller_p0_pll_mult(PLL_ADDR_CMD_CLK) \
          		-divide_by $::GLOBAL_ddr3_controller_p0_pll_div(PLL_ADDR_CMD_CLK) \
          		-phase $::GLOBAL_ddr3_controller_p0_pll_phase(PLL_ADDR_CMD_CLK) ]
        
        
        	set tracking_clock ${inst}|pll0|upll_memphy|auto_generated|pll1|clk\[3\]
          	
                set local_tracking_clk [ ddr3_controller_p0_get_or_add_clock_vseries \
          		-target $tracking_clock \
          		-suffix "tracking_clk" \
          		-source $pll_ref_clock \
          		-multiply_by $::GLOBAL_ddr3_controller_p0_pll_mult(PLL_AFI_HALF_CLK) \
          		-divide_by $::GLOBAL_ddr3_controller_p0_pll_div(PLL_AFI_HALF_CLK) \
          		-phase $::GLOBAL_ddr3_controller_p0_pll_phase(PLL_AFI_HALF_CLK) ]
        
        
        
        for { set i 0 } { $i < $::GLOBAL_ddr3_controller_p0_number_of_dqs_groups } { incr i } {
            set resync_x2_capture_clockname "${inst}_resync_x2_capture_${i}"
            create_generated_clock -name $resync_x2_capture_clockname -source [get_pins ${inst}|p0|umemphy|dq_ddio\[$i\].ubidir_dq_dqs|dq_ddio_io|clock_divider*_hr_clock.io_clkdiv|clk] -divide_by 2 [get_pins ${inst}|p0|umemphy|dq_ddio\[$i\].ubidir_dq_dqs|dq_ddio_io|clock_divider*_hr_clock.io_clkdiv|clkout] -add
        }
        
        # -------------------- #
        # -                  - #
        # --- WRITE CLOCKS --- #
        # -                  - #
        # -------------------- #
        
        # This is the DQS clock for Data Write analysis (micro model)
        foreach dqs_out_clock_struct $dqs_out_clocks {
        	array set dqs_out_clock $dqs_out_clock_struct
        	create_generated_clock -multiply_by 1 -master_clock [get_clocks $local_pll_write_clk] -source $pll_write_clock $dqs_out_clock(dst) -name $dqs_out_clock(dst)_OUT -add
        
        	# Clock Uncertainty is accounted for by the ...pathjitter parameters
        	set_clock_uncertainty -to [ get_clocks $dqs_out_clock(dst)_OUT ] 0
        }
        
        # This is the DQS#clock for Data Write analysis (micro model)
        foreach dqsn_out_clock_struct $dqsn_out_clocks {
        	array set dqsn_out_clock $dqsn_out_clock_struct
        	create_generated_clock -multiply_by 1 -invert -master_clock [get_clocks $local_pll_write_clk] -source $pll_write_clock $dqsn_out_clock(dst) -name $dqsn_out_clock(dst)_OUT -add
        
        	# Clock Uncertainty is accounted for by the ...pathjitter parameters
        	set_clock_uncertainty -to [ get_clocks $dqsn_out_clock(dst)_OUT ] 0
        }
	
        ##################
        #                #
        # READ DATA PATH #
        #                #
        ##################

        foreach { dqs_pin } $dqs_pins { dq_pins } $q_groups {
        	foreach { dq_pin } $dq_pins {
        		set_max_skew -from [get_ports $dq_pin] 0.1
        	}
        }
        
        ###################
        #                 #
        # WRITE DATA PATH #
        #                 #
        ###################
        
        foreach { dqs_pin } $dqs_pins { dq_pins } $q_groups {
        	foreach { dq_pin } $dq_pins {
        		# Specifies the minimum delay difference between the DQS pin and the DQ pins:
        		set_output_delay -min $data_output_min_delay -clock [get_clocks ${dqs_pin}_OUT ] [get_ports $dq_pin] -add_delay
        
        		# Specifies the maximum delay difference between the DQS pin and the DQ pins:
        		set_output_delay -max $data_output_max_delay -clock [get_clocks ${dqs_pin}_OUT ] [get_ports $dq_pin] -add_delay
        	}
        }
        
        foreach { dqsn_pin } $dqsn_pins { dq_pins } $q_groups {
        	foreach { dq_pin } $dq_pins {
        		# Specifies the minimum delay difference between the DQS#pin and the DQ pins:
        		set_output_delay -min $data_output_min_delay -clock [get_clocks ${dqsn_pin}_OUT ] [get_ports $dq_pin] -add_delay
        
        		# Specifies the maximum delay difference between the DQS#pin and the DQ pins:
        		set_output_delay -max $data_output_max_delay -clock [get_clocks ${dqsn_pin}_OUT ] [get_ports $dq_pin] -add_delay
        	}
        }
        
        foreach dqs_out_clock_struct $dqs_out_clocks {
        	array set dqs_out_clock $dqs_out_clock_struct
        
        	if { [string length $dqs_out_clock(dm_pin)] > 0 } {
        		# Specifies the minimum delay difference between the DQS and the DM pins:
        		set_output_delay -min $data_output_min_delay -clock [get_clocks $dqs_out_clock(dst)_OUT ] [get_ports $dqs_out_clock(dm_pin)] -add_delay
        
        		# Specifies the maximum delay difference between the DQS and the DM pins:
        		set_output_delay -max $data_output_max_delay -clock [get_clocks $dqs_out_clock(dst)_OUT ] [get_ports $dqs_out_clock(dm_pin)] -add_delay
        	}
        }
        
        foreach dqsn_out_clock_struct $dqsn_out_clocks {
        	array set dqsn_out_clock $dqsn_out_clock_struct
        
        	if { [string length $dqsn_out_clock(dm_pin)] > 0 } {
        		# Specifies the minimum delay difference between the DQS and the DM pins:
        		set_output_delay -min $data_output_min_delay -clock [get_clocks $dqsn_out_clock(dst)_OUT ] [get_ports $dqsn_out_clock(dm_pin)] -add_delay
        
        		# Specifies the maximum delay difference between the DQS and the DM pins:
        		set_output_delay -max $data_output_max_delay -clock [get_clocks $dqsn_out_clock(dst)_OUT ] [get_ports $dqsn_out_clock(dm_pin)] -add_delay
        	}
        }
        
        ###################
        ###
        ##DQS vs CK PATH #
        ###
        ###################
        
        foreach { ck_pin } $ck_pins { 
        	set_output_delay -add_delay -clock [get_clocks $ck_pin] -max [ddr3_controller_p0_round_3dp [expr $t(CK) - $t(DQSS)*$t(CK) - $board(minCK_DQS_skew) ]] $dqs_pins
        	set_output_delay -add_delay -clock [get_clocks $ck_pin] -min [ddr3_controller_p0_round_3dp [expr $t(DQSS)*$t(CK) - $board(maxCK_DQS_skew) ]] $dqs_pins
        	set_false_path -fall_from [get_clocks $local_pll_write_clk ] -to [get_ports $dqs_pins]
        }
        
        ############
        #          #
        # A/C PATH #
        #          #
        ############
        
        foreach { ck_pin } $ck_pins {
        	# ac_pins can contain input ports such as mem_err_out_n
        	# Loop through each ac pin to make sure we only apply set_output_delay to output ports
        	foreach { ac_pin } $ac_pins {
        		set ac_port [ get_ports $ac_pin ]
        		if {[get_collection_size $ac_port] > 0} {
        			if [ get_port_info -is_output_port $ac_port ] {
        				# Specifies the minimum delay difference between the DQS pin and the address/control pins:
        				set_output_delay -min $ac_min_delay -clock [get_clocks $ck_pin] $ac_port -add_delay
        
        				# Specifies the maximum delay difference between the DQS pin and the address/control pins:
        				set_output_delay -max $ac_max_delay -clock [get_clocks $ck_pin] $ac_port -add_delay
        			}
        		}
        	}
        }
        
#########################
#                       #
#  FALSE PATH           #
#                       #
#########################

        foreach { dqs_pin } $dqs_pins { dq_pins } $q_groups {
        	foreach { dq_pin } $dq_pins {
        		        set_false_path -from [get_registers *dq_ddio_io*oe_path_enhanced_ddr.fr_oe_data_ddio~DFF*] -to [get_ports $dq_pin]
        	}
        }
        
        set_false_path -rise_from [ get_clocks ${local_pll_write_clk} ] -to [ get_ports $ac_pins ]

        foreach { pin } [concat $dqs_pins $dqsn_pins $ck_pins $ckn_pins $reset_pins] {
        	set_max_skew -to [get_ports $pin] 0.1
        }

#########################
#                       #
# FITTER OVERCONSTRAINT #
#                       #
#########################

        if ($fit_flow) {
        set_max_delay -from [get_registers *hr_to_fr*] -to [get_registers *out_path_enhanced_ddr*] 2.8
        set_max_delay -from [get_registers *hr_to_fr*] -to [get_registers *oe_path_enhanced_ddr*] 3.3

        set_min_delay -from [get_registers *input*] -to [get_registers *dq_r*] 0.1
        }

}

######################
#                    #
# REPORT DDR COMMAND #
#                    #
######################

add_ddr_report_command "source [list [file join [file dirname [info script]] ${::GLOBAL_ddr3_controller_p0_corename}_report_timing.tcl]]"

