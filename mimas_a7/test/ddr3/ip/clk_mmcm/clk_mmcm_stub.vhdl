-- Copyright 1986-2022 Xilinx, Inc. All Rights Reserved.
-- --------------------------------------------------------------------------------
-- Tool Version: Vivado v.2022.1 (win64) Build 3526262 Mon Apr 18 15:48:16 MDT 2022
-- Date        : Sat Aug 13 20:18:03 2022
-- Host        : the-executive running 64-bit major release  (build 9200)
-- Command     : write_vhdl -force -mode synth_stub
--               c:/msys64/home/ferris/dev/projects/xenowing/mimas_a7/test/ddr3/ip/clk_mmcm/clk_mmcm_stub.vhdl
-- Design      : clk_mmcm
-- Purpose     : Stub declaration of top-level module interface
-- Device      : xc7a50tfgg484-1
-- --------------------------------------------------------------------------------
library IEEE;
use IEEE.STD_LOGIC_1164.ALL;

entity clk_mmcm is
  Port ( 
    clk_200 : out STD_LOGIC;
    reset : in STD_LOGIC;
    locked : out STD_LOGIC;
    clk_in1 : in STD_LOGIC
  );

end clk_mmcm;

architecture stub of clk_mmcm is
attribute syn_black_box : boolean;
attribute black_box_pad_pin : string;
attribute syn_black_box of stub : architecture is true;
attribute black_box_pad_pin of stub : architecture is "clk_200,reset,locked,clk_in1";
begin
end;
