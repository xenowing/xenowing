set_property CFGBVS VCCO [current_design]
set_property CONFIG_VOLTAGE 3.3 [current_design]

set_property -dict { PACKAGE_PIN "H4" IOSTANDARD LVCMOS33 } [get_ports { input_clk_100 }];

set_property -dict { PACKAGE_PIN "M2" IOSTANDARD LVCMOS33 } [get_ports { reset }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name reset_dummy_clk -period 10
set_input_delay -clock reset_dummy_clk -min 0 [get_ports { reset }]
set_input_delay -clock reset_dummy_clk -max 1 [get_ports { reset }]
set_false_path -from [get_ports { reset }] -to [all_registers]

set_property -dict { PACKAGE_PIN "Y21" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { uart_tx }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name uart_tx_dummy_clk -period 10
set_output_delay -clock uart_tx_dummy_clk -min 0 [get_ports { uart_tx }]
set_output_delay -clock uart_tx_dummy_clk -max 1 [get_ports { uart_tx }]
set_false_path -from [all_registers] -to [get_ports { uart_tx }]

set_property -dict { PACKAGE_PIN "K17" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { success_led }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name success_led_dummy_clk -period 10
set_output_delay -clock success_led_dummy_clk -min 0 [get_ports { success_led }]
set_output_delay -clock success_led_dummy_clk -max 1 [get_ports { success_led }]
set_false_path -from [all_registers] -to [get_ports { success_led }]

set_property -dict { PACKAGE_PIN "J17" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { calibration_done_led }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name calibration_done_led_dummy_clk -period 10
set_output_delay -clock calibration_done_led_dummy_clk -min 0 [get_ports { calibration_done_led }]
set_output_delay -clock calibration_done_led_dummy_clk -max 1 [get_ports { calibration_done_led }]
set_false_path -from [all_registers] -to [get_ports { calibration_done_led }]

set_property -dict { PACKAGE_PIN "M16" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { error_led }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name error_led_dummy_clk -period 10
set_output_delay -clock error_led_dummy_clk -min 0 [get_ports { error_led }]
set_output_delay -clock error_led_dummy_clk -max 1 [get_ports { error_led }]
set_false_path -from [all_registers] -to [get_ports { error_led }]
