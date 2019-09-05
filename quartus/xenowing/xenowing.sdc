derive_pll_clocks -create_base_clocks
derive_clock_uncertainty

# Don't constrain async reset signal
set_false_path -from [get_ports {global_reset_n}] -to [all_registers]

# Don't constrain uart tx/rx
set_false_path -from [all_registers] -to [get_ports {uart_tx}]
set_false_path -from [get_ports {uart_rx}] -to [all_registers]

# Don't constrain HDMI inputs/outputs
# TODO: I think it's actually a pretty bad idea to not constrain the video data/sync signals, even though the I2C bus is probably OK
set_false_path -from [all_registers] -to [get_ports {hdmi_scl}]
set_false_path -from [get_ports {hdmi_scl}] -to [all_registers]
set_false_path -from [all_registers] -to [get_ports {hdmi_sda}]
set_false_path -from [get_ports {hdmi_sda}] -to [all_registers]
set_false_path -from [all_registers] -to [get_ports {hdmi_pixel_clk}]
set_false_path -from [all_registers] -to [get_ports {hdmi_vsync}]
set_false_path -from [all_registers] -to [get_ports {hdmi_hsync}]
set_false_path -from [all_registers] -to [get_ports {hdmi_data_enable}]
set_false_path -from [all_registers] -to [get_ports {hdmi_pixel_data*}]

# Don't constrain LED output
set_false_path -from [all_registers] -to [get_ports {leds_n*}]
