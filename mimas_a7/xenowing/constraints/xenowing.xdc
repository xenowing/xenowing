set_property CFGBVS VCCO [current_design]
set_property CONFIG_VOLTAGE 3.3 [current_design]

set_property -dict { PACKAGE_PIN "H4" IOSTANDARD LVCMOS33 } [get_ports { input_clk_100 }];

set_property -dict { PACKAGE_PIN "M2" IOSTANDARD LVCMOS33 } [get_ports { reset }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name reset_dummy_clk -period 10
set_input_delay -clock reset_dummy_clk -min 0 [get_ports { reset }]
set_input_delay -clock reset_dummy_clk -max 1 [get_ports { reset }]
set_false_path -from [get_ports { reset }] -to [all_registers]

set_property -dict { PACKAGE_PIN "Y21" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { tx }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name tx_dummy_clk -period 10
set_output_delay -clock tx_dummy_clk -min 0 [get_ports { tx }]
set_output_delay -clock tx_dummy_clk -max 1 [get_ports { tx }]
set_false_path -from [all_registers] -to [get_ports { tx }]

set_property -dict { PACKAGE_PIN "Y22" IOSTANDARD LVCMOS33 } [get_ports { rx }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name rx_dummy_clk -period 10
set_input_delay -clock rx_dummy_clk -min 0 [get_ports { rx }]
set_input_delay -clock rx_dummy_clk -max 1 [get_ports { rx }]
set_false_path -from [get_ports { rx }] -to [all_registers]

set_property -dict { PACKAGE_PIN "K17" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { leds[0] }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name leds0_dummy_clk -period 10
set_output_delay -clock leds0_dummy_clk -min 0 [get_ports { leds[0] }]
set_output_delay -clock leds0_dummy_clk -max 1 [get_ports { leds[0] }]
set_false_path -from [all_registers] -to [get_ports { leds[0] }]

set_property -dict { PACKAGE_PIN "J17" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { leds[1] }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name leds1_dummy_clk -period 10
set_output_delay -clock leds1_dummy_clk -min 0 [get_ports { leds[1] }]
set_output_delay -clock leds1_dummy_clk -max 1 [get_ports { leds[1] }]
set_false_path -from [all_registers] -to [get_ports { leds[1] }]

set_property -dict { PACKAGE_PIN "L14" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { leds[2] }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name leds2_dummy_clk -period 10
set_output_delay -clock leds2_dummy_clk -min 0 [get_ports { leds[2] }]
set_output_delay -clock leds2_dummy_clk -max 1 [get_ports { leds[2] }]
set_false_path -from [all_registers] -to [get_ports { leds[2] }]

set_property -dict { PACKAGE_PIN "L15" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { leds[3] }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name leds3_dummy_clk -period 10
set_output_delay -clock leds3_dummy_clk -min 0 [get_ports { leds[3] }]
set_output_delay -clock leds3_dummy_clk -max 1 [get_ports { leds[3] }]
set_false_path -from [all_registers] -to [get_ports { leds[3] }]

set_property -dict { PACKAGE_PIN "L16" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { leds[4] }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name leds4_dummy_clk -period 10
set_output_delay -clock leds4_dummy_clk -min 0 [get_ports { leds[4] }]
set_output_delay -clock leds4_dummy_clk -max 1 [get_ports { leds[4] }]
set_false_path -from [all_registers] -to [get_ports { leds[4] }]

set_property -dict { PACKAGE_PIN "K16" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { leds[5] }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name leds5_dummy_clk -period 10
set_output_delay -clock leds5_dummy_clk -min 0 [get_ports { leds[5] }]
set_output_delay -clock leds5_dummy_clk -max 1 [get_ports { leds[5] }]
set_false_path -from [all_registers] -to [get_ports { leds[5] }]

set_property -dict { PACKAGE_PIN "M15" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { leds[6] }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name leds6_dummy_clk -period 10
set_output_delay -clock leds6_dummy_clk -min 0 [get_ports { leds[6] }]
set_output_delay -clock leds6_dummy_clk -max 1 [get_ports { leds[6] }]
set_false_path -from [all_registers] -to [get_ports { leds[6] }]

set_property -dict { PACKAGE_PIN "M16" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { leds[7] }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name leds7_dummy_clk -period 10
set_output_delay -clock leds7_dummy_clk -min 0 [get_ports { leds[7] }]
set_output_delay -clock leds7_dummy_clk -max 1 [get_ports { leds[7] }]
set_false_path -from [all_registers] -to [get_ports { leds[7] }]
