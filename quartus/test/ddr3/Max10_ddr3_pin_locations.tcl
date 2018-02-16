
#set_location_assignment PIN_C22 -to mem_a[13]
set_location_assignment PIN_J14 -to mem_a[12]
set_location_assignment PIN_E20 -to mem_a[11]
set_location_assignment PIN_Y20 -to mem_a[10]
set_location_assignment PIN_E22 -to mem_a[9]
set_location_assignment PIN_D22 -to mem_a[8]
set_location_assignment PIN_B20 -to mem_a[7]
set_location_assignment PIN_E21 -to mem_a[6]
set_location_assignment PIN_F19 -to mem_a[5]
set_location_assignment PIN_C20 -to mem_a[4]
set_location_assignment PIN_U20 -to mem_a[3]
set_location_assignment PIN_A21 -to mem_a[2]
set_location_assignment PIN_D19 -to mem_a[1]
set_location_assignment PIN_V20 -to mem_a[0]


set_location_assignment PIN_W22 -to mem_ba[2]
set_location_assignment PIN_N18 -to mem_ba[1]
set_location_assignment PIN_V22 -to mem_ba[0]
set_location_assignment PIN_U19 -to mem_cas_n
set_location_assignment PIN_D18 -to mem_ck
set_location_assignment PIN_E18 -to mem_ck_n
set_location_assignment PIN_W20 -to mem_cke
set_location_assignment PIN_Y22 -to mem_cs_n
set_location_assignment PIN_T18 -to mem_dm[2]
set_location_assignment PIN_N19 -to mem_dm[1]
set_location_assignment PIN_J15 -to mem_dm[0]
set_location_assignment PIN_P20 -to mem_dq[23]
set_location_assignment PIN_P15 -to mem_dq[22]
set_location_assignment PIN_T19 -to mem_dq[21]
set_location_assignment PIN_R15 -to mem_dq[20]
set_location_assignment PIN_R20 -to mem_dq[19]
set_location_assignment PIN_P14 -to mem_dq[18]
set_location_assignment PIN_P19 -to mem_dq[17]
set_location_assignment PIN_R14 -to mem_dq[16]
set_location_assignment PIN_N20 -to mem_dq[15]
set_location_assignment PIN_L19 -to mem_dq[14]
set_location_assignment PIN_M15 -to mem_dq[13]
set_location_assignment PIN_L18 -to mem_dq[12]
set_location_assignment PIN_M14 -to mem_dq[11]
set_location_assignment PIN_M20 -to mem_dq[10]
set_location_assignment PIN_M18 -to mem_dq[9]
set_location_assignment PIN_L20 -to mem_dq[8]
set_location_assignment PIN_K19 -to mem_dq[7]
set_location_assignment PIN_H20 -to mem_dq[6]
set_location_assignment PIN_J20 -to mem_dq[5]
set_location_assignment PIN_H19 -to mem_dq[4]
set_location_assignment PIN_K18 -to mem_dq[3]
set_location_assignment PIN_H18 -to mem_dq[2]
set_location_assignment PIN_K20 -to mem_dq[1]
set_location_assignment PIN_J18 -to mem_dq[0]
set_location_assignment PIN_R18 -to mem_dqs[2]
set_location_assignment PIN_L14 -to mem_dqs[1]
set_location_assignment PIN_K14 -to mem_dqs[0]
set_location_assignment PIN_W19 -to mem_odt
set_location_assignment PIN_V18 -to mem_ras_n
set_location_assignment PIN_B22 -to mem_reset_n
set_location_assignment PIN_Y21 -to mem_we_n
set_location_assignment PIN_N14 -to pll_ref_clk
set_location_assignment PIN_D9 -to global_reset_n
set_location_assignment PIN_N15 -to "pll_ref_clk(n)"
#set_instance_assignment -name IO_STANDARD "SSTL-15" -to mem_a[13]
set_instance_assignment -name IO_STANDARD "DIFFERENTIAL 1.5-V SSTL" -to pll_ref_clk
#set_location_assignment PIN_U22 -to drv_status_fail
#set_location_assignment PIN_AA22 -to drv_status_pass
#set_location_assignment PIN_AA21 -to drv_status_test_complete
#set_instance_assignment -name IO_STANDARD "1.5 V" -to drv_status_fail
#set_instance_assignment -name IO_STANDARD "1.5 V" -to drv_status_pass
#set_instance_assignment -name IO_STANDARD "1.5 V" -to drv_status_test_complete