set_property CFGBVS VCCO [current_design]
set_property CONFIG_VOLTAGE 3.3 [current_design]

create_clock -name input_clk_100 -period 10 [get_ports { input_clk_100 }];
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

set_property -dict { PACKAGE_PIN "K17" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { led }];
# Dummy clock/delays to suppress timing warnings for async signal
create_clock -name led_dummy_clk -period 10
set_output_delay -clock led_dummy_clk -min 0 [get_ports { led }]
set_output_delay -clock led_dummy_clk -max 1 [get_ports { led }]
set_false_path -from [all_registers] -to [get_ports { led }]
