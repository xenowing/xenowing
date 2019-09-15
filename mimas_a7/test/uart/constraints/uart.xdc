set_property CFGBVS VCCO [current_design]
set_property CONFIG_VOLTAGE 3.3 [current_design]

create_clock -name clk -period 10 [get_ports { clk }];
set_property -dict { PACKAGE_PIN "H4" IOSTANDARD LVCMOS33 } [get_ports { clk }];

set_property -dict { PACKAGE_PIN "M2" IOSTANDARD LVCMOS33 } [get_ports { reset }];
# Dummy delay to suppress timing warnings
set_input_delay -clock clk 0 [get_ports { reset }]
set_false_path -from [get_ports { reset }] -to [all_registers]

set_property -dict { PACKAGE_PIN "Y21" IOSTANDARD LVCMOS33 SLEW FAST } [get_ports { tx }];
# TODO: These should be better understood/adjusted
set_output_delay -clock clk -min 1 [get_ports { tx }]
set_output_delay -clock clk -max -1 [get_ports { tx }]
