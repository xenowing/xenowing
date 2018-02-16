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


`timescale 1 ps / 1 ps

module max10emif_dcfifo #
    ( parameter
        numwords = 8,
        width = 24,
        widthu = 3
    )
    (
    data,
    rdclk,
    rdreq,
    wrclk,
    wrreq,
    q,
    aclr
);

    input   [width-1:0] data;
    input   rdclk;
    input   rdreq;
    input   wrclk;
    input   wrreq;
    output  [width-1:0] q;
    input   aclr;
    
    logic   [widthu-1:0] rdptr;
    logic   [widthu-1:0] wrptr;
    
    
    always_ff @ (posedge wrclk or posedge aclr)
        begin
            if (aclr)
                wrptr    <=    0;
            else
                wrptr    <=    wrptr + wrreq;
        end
    
    
    always_ff @ (posedge rdclk or posedge aclr)
        begin
            if (aclr)
                rdptr    <=    0;
            else
                rdptr    <=    rdptr + rdreq;
        end
    
    
    logic    [width-1:0]    fifo    [numwords-1:0];
    
    always_ff @ (posedge wrclk)
        fifo [wrptr]    <=    data;
    
    assign    q = fifo[rdptr];

endmodule
