// (C) 2001-2019 Intel Corporation. All rights reserved.
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


// ******
// sequencer_m10
// ******
//
// Max10 RTL Sequencer
//
// General Description
// -------------------
//
// This component replaces the NIOS to calibrate Max10 EMIF
// Max10 EMIF uses PLL clock to capture DQ data
//
//

`timescale 1 ps / 1 ps

module sequencer_m10 #
    ( parameter
        AVL_DATA_WIDTH          = 32,
        AVL_ADDR_WIDTH          = 20, 
        READ_VALID_FIFO_SIZE    = 16,
        HPS_PROTOCOL            = "DDR3",
        PHY_MGR_BASE            = 294912,
        RW_MGR_BASE             = 327680,
        PLL_MGR_BASE            = 360448,
        MEM_DQ_WIDTH            = 24
    )
    (
	clk,
	reset_n,
	address,
	write,
	writedata,
	read,
	readdata,
	waitrequest,
	pd_reset_n,
	pd_ack,
	pd_up,
	pd_down
);

    localparam  PRE_N_ACT_1 =   (HPS_PROTOCOL == "DDR3") ? 'h14 : ((HPS_PROTOCOL == "DDR2") ? 'h16 : 'h14);
    localparam  PRE_N_ACT_2 =   (HPS_PROTOCOL == "DDR3") ? 'hf  : ((HPS_PROTOCOL == "DDR2") ? 'hf  : 'hf );
    localparam  PRE_N_ACT_3 =   (HPS_PROTOCOL == "DDR3") ? 'h10 : ((HPS_PROTOCOL == "DDR2") ? 'h12 : 'h10);
    localparam  PRE_N_ACT_4 =   (HPS_PROTOCOL == "DDR3") ? 'hf  : ((HPS_PROTOCOL == "DDR2") ? 'hf  : 'hf );
    localparam  PRE_N_ACT_5 =   (HPS_PROTOCOL == "DDR3") ? 'h12 : ((HPS_PROTOCOL == "DDR2") ? 'h14 : 'h12);
    localparam  PRE_N_ACT_6 =   (HPS_PROTOCOL == "DDR3") ? 'hf  : ((HPS_PROTOCOL == "DDR2") ? 'h11 : 'hf );
    
    localparam  GRNTEE_WR_1 =   (HPS_PROTOCOL == "DDR3") ? 'h20 : ((HPS_PROTOCOL == "DDR2") ? 'h20 : 'h20);
    localparam  GRNTEE_WR_2 =   (HPS_PROTOCOL == "DDR3") ? 'h1c : ((HPS_PROTOCOL == "DDR2") ? 'h1d : 'h19);
    localparam  GRNTEE_WR_3 =   (HPS_PROTOCOL == "DDR3") ? 'h20 : ((HPS_PROTOCOL == "DDR2") ? 'h20 : 'h20);
    localparam  GRNTEE_WR_4 =   (HPS_PROTOCOL == "DDR3") ? 'h20 : ((HPS_PROTOCOL == "DDR2") ? 'h21 : 'h1d);
    localparam  GRNTEE_WR_5 =   (HPS_PROTOCOL == "DDR3") ? 'h2  : ((HPS_PROTOCOL == "DDR2") ? 'h2  : 'h2 );
    localparam  GRNTEE_WR_6 =   (HPS_PROTOCOL == "DDR3") ? 'h1a : ((HPS_PROTOCOL == "DDR2") ? 'h1b : 'h17);
    localparam  GRNTEE_WR_7 =   (HPS_PROTOCOL == "DDR3") ? 'h2  : ((HPS_PROTOCOL == "DDR2") ? 'h2  : 'h2 );
    localparam  GRNTEE_WR_8 =   (HPS_PROTOCOL == "DDR3") ? 'h1e : ((HPS_PROTOCOL == "DDR2") ? 'h1f : 'h1b);
    localparam  GRNTEE_WR_9 =   (HPS_PROTOCOL == "DDR3") ? 'h19 : ((HPS_PROTOCOL == "DDR2") ? 'h1a : 'h16);
    
    localparam  READ_CHK_1  =   (HPS_PROTOCOL == "DDR3") ? 'h10 : ((HPS_PROTOCOL == "DDR2") ? 'h10 : 'h10);
    localparam  READ_CHK_2  =   (HPS_PROTOCOL == "DDR3") ? 'h45 : ((HPS_PROTOCOL == "DDR2") ? 'h46 : 'h42);
    localparam  READ_CHK_3  =   (HPS_PROTOCOL == "DDR3") ? 'h10 : ((HPS_PROTOCOL == "DDR2") ? 'h10 : 'h10);
    localparam  READ_CHK_4  =   (HPS_PROTOCOL == "DDR3") ? 'h4b : ((HPS_PROTOCOL == "DDR2") ? 'h4c : 'h48);
    localparam  READ_CHK_5  =   (HPS_PROTOCOL == "DDR3") ? 'h1  : ((HPS_PROTOCOL == "DDR2") ? 'h1  : 'h1 );
    localparam  READ_CHK_6  =   (HPS_PROTOCOL == "DDR3") ? 'h41 : ((HPS_PROTOCOL == "DDR2") ? 'h42 : 'h3e);
    localparam  READ_CHK_7  =   (HPS_PROTOCOL == "DDR3") ? 'h1  : ((HPS_PROTOCOL == "DDR2") ? 'h1  : 'h1 );
    localparam  READ_CHK_8  =   (HPS_PROTOCOL == "DDR3") ? 'h41 : ((HPS_PROTOCOL == "DDR2") ? 'h42 : 'h3e);
    localparam  READ_CHK_9  =   (HPS_PROTOCOL == "DDR3") ? 'h0  : ((HPS_PROTOCOL == "DDR2") ? 'h0  : 'h0 );
    localparam  READ_CHK_10 =   (HPS_PROTOCOL == "DDR3") ? 'h41 : ((HPS_PROTOCOL == "DDR2") ? 'h42 : 'h3e);
    localparam  READ_CHK_11 =   (HPS_PROTOCOL == "DDR3") ? 'h3e : ((HPS_PROTOCOL == "DDR2") ? 'h3f : 'h3b);
    
    localparam  READ_LATENCY    =   5'h16;
    localparam  DISABLE_FILTER  =    1'b0;
    localparam  DISABLE_PD      =    1'b0; 

	input                              clk;
	input                              reset_n;
	output [AVL_ADDR_WIDTH - 1:0]      address;
	output                             write;
	output [AVL_DATA_WIDTH - 1:0]      writedata;
	output                             read;
	input [AVL_DATA_WIDTH - 1:0]       readdata;
	input                              waitrequest;
	output                             pd_reset_n;
	output                             pd_ack;
	input                              pd_up;
	input                              pd_down;
	
	logic [AVL_ADDR_WIDTH - 1:0]       address;
	logic                              write;
	logic [AVL_DATA_WIDTH - 1:0]       writedata;
	logic                              read;
	logic                              pd_reset_n;
	logic                              pd_ack;
	
	logic [5:0]    seq_state;
	logic [AVL_ADDR_WIDTH - 1:0]       main_address;
	logic                              main_write;
	logic [AVL_DATA_WIDTH - 1:0]       main_writedata;
	logic                              main_read;
	logic [23:0]   readdata_reg;
	logic [23:0]   best_comp_result;
	logic          data_mismatch;
	logic [2:0]    vfifo;
	logic [7:0]    count;
	logic [7:0]    valid_phase;
	logic          reverse;
	logic          finding_window;
	logic          window_found;
	logic          latency_calib;
	logic [1:0]    grp;
	
	logic    start_ddr3_init;
	logic    ddr3_init_done;
	logic    start_ddr2_init;
	logic    ddr2_init_done;
	logic    start_lpddr2_init;
	logic    lpddr2_init_done;
	logic    start_write;
	logic    write_done;
	logic    start_user_mr;
	logic    user_mr_done;
	logic    pd_start;
	logic    cal_init_req;
	
	logic          wr_and_not_waitrequest;
	
	assign    wr_and_not_waitrequest    =    main_write & ~waitrequest;
	
    always_ff @(posedge clk, negedge reset_n)
        begin
            if (!reset_n)
                begin
                    seq_state         <=  0;
                    main_address      <=  '0;
                    main_write        <=  0;
                    main_writedata    <=  '0;
                    main_read         <=  0;
                    readdata_reg      <=  '0;
                    best_comp_result  <=  '0;
                    data_mismatch     <=  0;
                    vfifo             <=  '0;
                    count             <=  0;
                    valid_phase       <=  0;
                    reverse           <=  0;
                    finding_window    <=  0;
                    window_found      <=  0;
                    latency_calib     <=  0;
                    start_ddr3_init   <=  0;
                    start_ddr2_init   <=  0;
                    start_lpddr2_init <=  0;
                    start_write       <=  0;
                    start_user_mr     <=  0;
                    pd_start          <=  0;
                    grp               <=  '0;
                end
            else
                case(seq_state)
                    0   :
                        begin
                            best_comp_result <= {24{1'b1}};
                            if (HPS_PROTOCOL == "DDR3")
                                seq_state       <=  1;
                            else if (HPS_PROTOCOL == "DDR2")
                                seq_state       <=  2;
                            else // LPDDR2
                                seq_state       <=  3;
                        end
                    
                    // start DDR3 initialization sequence
                    1   :
                        begin
                            start_ddr3_init     <=  1;
                            if (ddr3_init_done)
                                begin
                                    start_ddr3_init <=  0;
                                    seq_state       <=  4;
                                end
                        end
                    // end of DDR3 initialization sequence
                    
                    // start DDR2 initialization sequence
                    2   :
                        begin
                            start_ddr2_init     <=  1;
                            if (ddr2_init_done)
                                begin
                                    start_ddr2_init <=  0;
                                    seq_state       <=  4;
                                end
                        end
                    // end of DDR2 initialization sequence
                    
                    // start LPDDR2 initialization sequence
                    3   :
                        begin
                            start_lpddr2_init   <=  1;
                            if (lpddr2_init_done)
                                begin
                                    start_lpddr2_init   <=  0;
                                    seq_state       <=  4;
                                end
                        end
                    // end of LPDDR2 initialization sequence
                    
                    // precharge and activate then guarantee write
                    4   :
                        begin
                            start_write   <=  1;
                            if (write_done)
                                begin
                                    start_write     <=  0;
                                    seq_state       <=  5;
                                end
                        end
                    
                    // read data checking routine
                    5   : write_task_rw ('h201     ,READ_CHK_1  ,6  );
                    6   : write_task_rw ('h301     ,READ_CHK_2  ,7  );
                    7   : write_task_rw ('h202     ,READ_CHK_3  ,8  );
                    8   : write_task_rw ('h302     ,READ_CHK_4  ,9  );
                    9   : write_task_rw ('h200     ,READ_CHK_5  ,10 );
                    10  : write_task_rw ('h300     ,READ_CHK_6  ,11 );
                    11  : write_task_rw ('h203     ,READ_CHK_7  ,12 );
                    12  : write_task_rw ('h303     ,READ_CHK_8  ,13 );
                    13  : write_task_ph ('h2       ,'h0         ,14 ); // PHY manager, read fifo reset
                    14  : write_task_rw ('h400     ,READ_CHK_9  ,15 );
                    15  : write_task_rw (grp       ,READ_CHK_10 ,16 );
                    16  :
                        begin
                            main_read    <=  1;
                            main_address <=  RW_MGR_BASE[AVL_ADDR_WIDTH - 1:0];//
                            if (main_read && !waitrequest)
                                begin
                                    main_read       <=  0;
                                    main_address    <=  '0;
                                    seq_state       <=  38;
                                    if (grp == 2)
                                        readdata_reg[23:16]    <=  readdata[7:0];
                                    else if (grp == 1)
                                        readdata_reg[15:8]    <=  readdata[7:0];
                                    else
                                        readdata_reg[7:0]    <=  readdata[7:0];
                                end
                        end
                    38  :
                        begin
                            if (MEM_DQ_WIDTH == 24)
                                begin
                                    if (grp == 2)
                                        begin
                                            grp         <=  0;
                                            seq_state   <=  17;
                                        end
                                    else
                                        begin
                                            grp         <=  grp + 1'b1;
                                            seq_state   <=  5;
                                        end
                                end
                            else if (MEM_DQ_WIDTH == 16)
                                begin
                                    if (grp == 1)
                                        begin
                                            grp         <=  0;
                                            seq_state   <=  17;
                                        end
                                    else
                                        begin
                                            grp         <=  grp + 1'b1;
                                            seq_state   <=  5;
                                        end
                                end
                            else 
                                seq_state   <=  17;
                        end
                    17  : write_task_rw ('h0       ,READ_CHK_11 ,18 );
                    18  :
                        begin
                            if (readdata_reg != 0)
                                data_mismatch   <=  1;
                            else
                                data_mismatch   <=  0;
                            
                            if (readdata_reg[23:16] < best_comp_result[23:16])
                                best_comp_result[23:16]    <=  readdata_reg[23:16];
                            
                            if (readdata_reg[15:8] < best_comp_result[15:8])
                                best_comp_result[15:8]    <=  readdata_reg[15:8];
                            
                            if (readdata_reg[7:0] < best_comp_result[7:0])
                                best_comp_result[7:0]    <=  readdata_reg[7:0];
                                
                            if (latency_calib)
                                seq_state   <=  29;
                            else if (finding_window)
                                seq_state   <=  26;
                            else if (reverse)
                                seq_state   <=  24;
                            else
                                seq_state   <=  19;
                        end
                    // end read data checking routine
                    
                    // find first working phase
                    19  :
                        begin
                            if (data_mismatch) // data mismatch
                                begin
                                    if (count == 150) // 57x2 is the calculated steps for 167Mhz
                                        seq_state   <=  22; // give up, fail calibration
                                    else if (vfifo == 7) // increment PLL phase
                                        begin
                                            write_task_capture_clock (1         ,20 );
                                            
                                            if (wr_and_not_waitrequest)
                                                count       <=  count + 1'b1;
                                        end
                                    else
                                        seq_state   <=  20;
                                end
                            else // data match! then reverse phase
                                begin
                                    $display("          --- SEQUENCER FOUND FIRST WORKING PHASE --- ");
                                    reverse             <=  1;
                                    seq_state           <=  5 ;
                                end
                        end
                    20  :
                        begin
                            vfifo       <=  vfifo + 1'b1;
                            seq_state   <=  21;
                        end
                    21  : write_task_ph ('h1       ,'hff        ,5  ); // increment vfifo for all groups
                    // algorithm ends here
                    
                    // PHY manager, cal fail
                    22  : write_task_ph ('h1003    ,'h2         ,36 );
                    
                    // reverse phase until fail, useful if already in a working phase on start
                    24 :
                        begin
                            if (data_mismatch) //data mismatch, increment and move to finding window
                                begin
                                    reverse     <=  0;
                                    write_task_capture_clock (1         ,25 );
                                end
                            else // data match, decrement until it fails
                                write_task_capture_clock (0         ,5  );
                        end
                    
                    // finding window, sweep till fail
                    25  :
                        begin
                            $display("          --- START FINDING WINDOW --- ");
                            finding_window      <=  1;
                            count               <=  1;
                            seq_state           <=  5 ;
                        end
                    26  :
                        begin
                            if (data_mismatch) // data mismatch
                                begin
                                    if (!window_found) 
                                        write_task_capture_clock (1         ,5  ); 
                                    else
                                        begin
                                            $display("          --- TOTAL VALID PHASE %d --- ",count - 1);
                                            valid_phase     <=  count-1'b1;
                                            finding_window  <=  0;
                                            count           <=  count/2'd2; // formula to move the correct number of phase
                                            $display("          --- REVERSE TO CENTER %d PHASE(S)--- ",((count - 2)/2)+2);
                                            seq_state       <=  27;
                                        end
                                end
                            else
                                begin
                                    window_found    <=  1;
                                    write_task_capture_clock (1         ,5  ); // increment PLL phase
                                    
                                    if (wr_and_not_waitrequest)
                                        count           <=  count+1'b1;
                                end
                        end
                    
                    // going to the center of window after sweep
                    27  :
                        begin
                            if (count != 0)
                                begin
                                    write_task_capture_clock (0         ,27 ); // decrement PLL phase
                                    
                                    if (wr_and_not_waitrequest)
                                        count       <=  count-1'b1;
                                end
                            else
                                seq_state   <=  31;
                        end
                    
                    // calibration done
                    31  : write_task_rw ('h0       ,PRE_N_ACT_1 ,32 ); // precharge
                    32  : write_task_ph ('h2       ,'h0         ,33 ); // PHY manager, read fifo reset
                    33  :
                        begin
                            start_user_mr   <=  1;
                            if (user_mr_done)
                                begin
                                    start_user_mr   <=  0;
                                    seq_state       <=  34;
                                end
                        end
                    
                    // PHY manager, mux sel
                    34  : write_task_ph ('h1002    ,'h0         ,35 );
                    // PHY manager, cal success
                    35  : write_task_ph ('h1003    ,'h1         ,36 );
                    // PHY manager, debug info reports window size, best comparison data
                    36  : write_task_ph ('h1004    ,{best_comp_result,valid_phase}         ,37 );
                    // calibration done
                    37  : 
                        begin
                            if (pd_start && cal_init_req)
                                begin
                                    pd_start <=  0;
                                    seq_state   <=  39;
                                    
                                    readdata_reg      <=  '0;
                                    best_comp_result  <=  '0;
                                    data_mismatch     <=  0;
                                    vfifo             <=  '0;
                                    count             <=  0;
                                    valid_phase       <=  0;
                                    reverse           <=  0;
                                    finding_window    <=  0;
                                    window_found      <=  0;
                                    latency_calib     <=  0;
                                    grp               <=  '0;
                                end
                            else
                                begin
                                    pd_start <=  1;
                                end
                        end
                    39  : write_task_ph ('h1003    ,'h0         ,40 );
                    40  : write_task_ph ('h1002    ,'h1         ,0  );
                endcase
        end
        
task write_task_rw;
    input [AVL_ADDR_WIDTH - 1:0] task_address;
    input [AVL_DATA_WIDTH - 1:0] data;
    input [5:0] next_state;
    
    begin
        main_write       <=  1;
        main_address     <=  RW_MGR_BASE[AVL_ADDR_WIDTH - 1:0] + {task_address[AVL_ADDR_WIDTH - 3:0],2'b0};
        main_writedata   <=  data;
        if (wr_and_not_waitrequest)
            begin
                main_write       <=  0;
                main_address     <=  '0;
                main_writedata   <=  '0;
                seq_state   <=  next_state;
            end
    end
endtask

task write_task_ph;
    input [AVL_ADDR_WIDTH - 1:0] task_address;
    input [AVL_DATA_WIDTH - 1:0] data;
    input [5:0] next_state;
    
    begin
        main_write       <=  1;
        main_address     <=  PHY_MGR_BASE[AVL_ADDR_WIDTH - 1:0] + {task_address[AVL_ADDR_WIDTH - 3:0],2'b0};
        main_writedata   <=  data;
        if (wr_and_not_waitrequest)
            begin
                main_write       <=  0;
                main_address     <=  '0;
                main_writedata   <=  '0;
                seq_state   <=  next_state;
            end
    end
endtask

task write_task_capture_clock;
    input data;
    input [5:0] next_state;
    
    begin
        main_write       <=  1;
        main_address     <=  PLL_MGR_BASE[AVL_ADDR_WIDTH - 1:0];
        main_writedata   <=  data;
        if (wr_and_not_waitrequest)
            begin
                main_write       <=  0;
                main_address     <=  '0;
                main_writedata   <=  '0;
                seq_state   <=  next_state;
            end
    end
endtask
    
	logic [4:0]    ddr3_init_state;
	logic [AVL_ADDR_WIDTH - 1:0]       ddr3_init_address;
	logic                              ddr3_init_write;
	logic [AVL_DATA_WIDTH - 1:0]       ddr3_init_writedata;

generate
    if (HPS_PROTOCOL == "DDR3")
        begin
            task ddr3_init_write_task_rw;
                input [AVL_ADDR_WIDTH - 1:0] task_address;
                input [AVL_DATA_WIDTH - 1:0] data;
                input [4:0] next_state;
            
                begin
                    ddr3_init_write       <=  1;
                    ddr3_init_address     <=  RW_MGR_BASE[AVL_ADDR_WIDTH - 1:0] + {task_address[AVL_ADDR_WIDTH - 3:0],2'b0};
                    ddr3_init_writedata   <=  data;
                    if (ddr3_init_write && !waitrequest)
                        begin
                            ddr3_init_write       <=  0;
                            ddr3_init_address     <=  '0;
                            ddr3_init_writedata   <=  '0;
                            ddr3_init_state   <=  next_state;
                        end
                end
            endtask            
            always_ff @(posedge clk, negedge reset_n)
                begin
                    if (!reset_n)
                        begin
                            ddr3_init_state       <=  0;
                            ddr3_init_done        <=  0;
                            ddr3_init_address     <=  '0;
                            ddr3_init_write       <=  0;
                            ddr3_init_writedata   <=  '0;
                        end
                    else
                        case(ddr3_init_state)
                            0   : if (start_ddr3_init) ddr3_init_state <=  1;
                            // start DDR3 initialization sequence
                            //                   address    data         next_state
                            1   : ddr3_init_write_task_rw ('h800     ,'h0         ,2  );
                            2   : ddr3_init_write_task_rw ('h500     ,'hfe        ,3  );
                            3   : ddr3_init_write_task_rw ('h500     ,'h0         ,4  );
                            4   : ddr3_init_write_task_rw ('h200     ,'hff         ,5  );
                            5   : ddr3_init_write_task_rw ('h201     ,'hff         ,6  );
                            6   : ddr3_init_write_task_rw ('h300     ,'h4f        ,7  );
                            7   : ddr3_init_write_task_rw ('h301     ,'h50        ,8  );
                            8   : ddr3_init_write_task_rw ('h0       ,'h4f        ,9  ); // delay?
                            9   : ddr3_init_write_task_rw ('h200     ,'hff         ,10 );
                            10  : ddr3_init_write_task_rw ('h201     ,'hff         ,11 );
                            11  : ddr3_init_write_task_rw ('h300     ,'h54        ,12 );
                            12  : ddr3_init_write_task_rw ('h301     ,'h55        ,13 );
                            13  : ddr3_init_write_task_rw ('h0       ,'h54        ,14 ); // reset goes high
                            14  : ddr3_init_write_task_rw ('h201     ,'h7c        ,15 ); 
                            15  : ddr3_init_write_task_rw ('h301     ,'h5c        ,16 ); 
                            16  : ddr3_init_write_task_rw ('h0       ,'h5c        ,17 ); /// cke goes high
                            17  : ddr3_init_write_task_rw ('h0       ,'h7         ,18 ); // MR bank 2
                            18  : ddr3_init_write_task_rw ('h0       ,'h9         ,19 ); // MR bank 3
                            19  : ddr3_init_write_task_rw ('h0       ,'h5         ,20 ); // MR bank 1
                            20  : ddr3_init_write_task_rw ('h0       ,'h3         ,21 ); // DLL reset
                            21  : ddr3_init_write_task_rw ('h0       ,'hb         ,22 ); // ZQ cal
                            // end of DDR3 initialization sequence
                            22  :
                                begin
                                    if (cal_init_req)
                                        begin
                                            ddr3_init_done      <=  0;
                                            ddr3_init_state     <=  0;
                                        end
                                    else
                                        ddr3_init_done    <=  1;
                                end
                        endcase
                end
            
        end
    else
        begin
            assign  ddr3_init_done          =   0;
            assign  ddr3_init_address       =   '0;
            assign  ddr3_init_write         =   0;
            assign  ddr3_init_writedata     =   '0;
        end
endgenerate
        
    logic [4:0]    ddr2_init_state;
	logic [AVL_ADDR_WIDTH - 1:0]       ddr2_init_address;
	logic                              ddr2_init_write;
	logic [AVL_DATA_WIDTH - 1:0]       ddr2_init_writedata;
	logic [6:0]    count2;

generate
    if (HPS_PROTOCOL == "DDR2")
        begin
                task ddr2_init_write_task_rw;
                input [AVL_ADDR_WIDTH - 1:0] task_address;
                input [AVL_DATA_WIDTH - 1:0] data;
                input [4:0] next_state;
            
                begin
                    ddr2_init_write       <=  1;
                    ddr2_init_address     <=  RW_MGR_BASE[AVL_ADDR_WIDTH - 1:0] + {task_address[AVL_ADDR_WIDTH - 3:0],2'b0};
                    ddr2_init_writedata   <=  data;
                    if (ddr2_init_write && !waitrequest)
                        begin
                            ddr2_init_write       <=  0;
                            ddr2_init_address     <=  '0;
                            ddr2_init_writedata   <=  '0;
                            ddr2_init_state   <=  next_state;
                        end
                end
            endtask
            always_ff @(posedge clk, negedge reset_n)
                begin
                    if (!reset_n)
                        begin
                            ddr2_init_state       <=  0;
                            ddr2_init_done        <=  0;
                            ddr2_init_address     <=  '0;
                            ddr2_init_write       <=  0;
                            ddr2_init_writedata   <=  '0;
                            count2                <=  '0;
                        end
                    else
                        case(ddr2_init_state)
                            0   : if (start_ddr2_init) ddr2_init_state <=  1;
                            // start DDR2 initialization sequence
                            //                   address    data         next_state
                            1   : ddr2_init_write_task_rw ('h800     ,'h0         ,2  );
                            2   : ddr2_init_write_task_rw ('h500     ,'hfe        ,3  );
                            3   : ddr2_init_write_task_rw ('h200     ,'hff        ,4  );
                            4   : ddr2_init_write_task_rw ('h201     ,'hff        ,5  );
                            5   : ddr2_init_write_task_rw ('h300     ,'h50        ,6  );
                            6   : ddr2_init_write_task_rw ('h301     ,'h51        ,7  );
                            7   : ddr2_init_write_task_rw ('h0       ,'h50        ,8  ); // delay?
                            8   : ddr2_init_write_task_rw ('h0       ,'h1         ,22 ); // CKE goes high
                            22  :
                                begin
                                    if (count2 == 100)
                                        begin
                                            ddr2_init_state   <=  9;
                                            count2            <=  0;
                                        end
                                    else
                                        count2      <=  count2 + 1'b1;
                                end
                            9   : ddr2_init_write_task_rw ('h0       ,'h16        ,10 ); // precharge
                            10  : ddr2_init_write_task_rw ('h0       ,'hd         ,11 ); // MR bank 2
                            11  : ddr2_init_write_task_rw ('h0       ,'hf         ,12 ); // MR bank 3
                            12  : ddr2_init_write_task_rw ('h0       ,'h9         ,13 ); // MR bank 1
                            13  : ddr2_init_write_task_rw ('h0       ,'h7         ,14 ); // MR bank 0
                            14  : ddr2_init_write_task_rw ('h0       ,'h16        ,15 ); // precharge
                            15  : ddr2_init_write_task_rw ('h0       ,'h18        ,16 ); // refresh
                            16  :
                                begin
                                    if (count2 == 100)
                                        begin
                                            ddr2_init_state   <=  17;
                                            count2            <=  0;
                                        end
                                    else
                                        count2      <=  count2 + 1'b1;
                                end
                            17  : ddr2_init_write_task_rw ('h0       ,'h18        ,23 ); // refresh
                            23  :
                                begin
                                    if (count2 == 100)
                                        begin
                                            ddr2_init_state   <=  18;
                                            count2            <=  0;
                                        end
                                    else
                                        count2      <=  count2 + 1'b1;
                                end
                            18  : ddr2_init_write_task_rw ('h0       ,'h5         ,19 ); // MR bank 0
                            19  : ddr2_init_write_task_rw ('h0       ,'hb         ,20 ); // MR bank 1
                            20  : ddr2_init_write_task_rw ('h0       ,'h9         ,21 ); // MR bank 1
                            // end of DDR2 initialization sequence
                            21  :
                                begin
                                    if (cal_init_req)
                                        begin
                                            ddr2_init_done      <=  0;
                                            ddr2_init_state     <=  0;
                                        end
                                    else
                                        ddr2_init_done    <=  1;
                                end
                        endcase
                end
            
        end
    else
        begin
            assign  ddr2_init_done          =   0;
            assign  ddr2_init_address       =   '0;
            assign  ddr2_init_write         =   0;
            assign  ddr2_init_writedata     =   '0;
        end
endgenerate
        
    logic [3:0]    lpddr2_init_state;
	logic [AVL_ADDR_WIDTH - 1:0]       lpddr2_init_address;
	logic                              lpddr2_init_write;
	logic [AVL_DATA_WIDTH - 1:0]       lpddr2_init_writedata;
	logic [7:0]    count4;
	logic [7:0]    count5;

generate
    if (HPS_PROTOCOL == "LPDDR2")
        begin
            task lpddr2_init_write_task_rw;
                input [AVL_ADDR_WIDTH - 1:0] task_address;
                input [AVL_DATA_WIDTH - 1:0] data;
                input [3:0] next_state;
                begin
                    lpddr2_init_write       <=  1;
                    lpddr2_init_address     <=  RW_MGR_BASE[AVL_ADDR_WIDTH - 1:0] + {task_address[AVL_ADDR_WIDTH - 3:0],2'b0};
                    lpddr2_init_writedata   <=  data;
                    if (lpddr2_init_write && !waitrequest)
                        begin
                            lpddr2_init_write       <=  0;
                            lpddr2_init_address     <=  '0;
                            lpddr2_init_writedata   <=  '0;
                            lpddr2_init_state   <=  next_state;
                        end
                end
            endtask
            
            always_ff @(posedge clk, negedge reset_n)
                begin
                    if (!reset_n)
                        begin
                            lpddr2_init_state       <=  0;
                            lpddr2_init_done        <=  0;
                            lpddr2_init_address     <=  '0;
                            lpddr2_init_write       <=  0;
                            lpddr2_init_writedata   <=  '0;
                            count4                  <=  '0;
                            count5                  <=  '0;
                        end
                    else
                        case(lpddr2_init_state)
                            0   : if (start_lpddr2_init) lpddr2_init_state <=  1;
                            // start LPDDR2 initialization sequence
                            //                   address    data         next_state
                            1   : lpddr2_init_write_task_rw ('h800     ,'h0         ,2  );
                            2   : lpddr2_init_write_task_rw ('h500     ,'hfe        ,3  );
                            3   : lpddr2_init_write_task_rw ('h200     ,'h10        ,4  );
                            4   : lpddr2_init_write_task_rw ('h300     ,'h4c        ,15  );
                            15  : 
                                begin
                                    if (&count4)
                                        begin
                                            count4 <=  0;
                                            if (&count5)
                                                begin
                                                    count5 <= 0;
                                                    lpddr2_init_state   <=  5;
                                                end
                                            else
                                                count5 <=  count5 + 1'b1;
                                        end
                                    else
                                        count4 <=  count4 + 1'b1;
                                end 
                            5   : lpddr2_init_write_task_rw ('h0       ,'h4c        ,12 ); // CKE goes high
                            12  : //200us delay
                                begin
                                    if (&count4)
                                        begin
                                            count4            <=  0;
                                            if (&count5)
                                                begin
                                                    lpddr2_init_state   <=  6;
                                                    count5            <=  0;
                                                end
                                            else
                                                count5      <=  count5 + 1'b1;
                                        end
                                    else
                                        count4      <=  count4 + 1'b1;
                                end
                            6   : lpddr2_init_write_task_rw ('h0       ,'hd         ,13 ); // MRW reset
                            13  : //10us delay
                                begin
                                    if (count4 == 100)
                                        begin
                                            count4            <=  0;
                                            if (count5 == 10)
                                                begin
                                                    lpddr2_init_state   <=  7;
                                                    count5            <=  0;
                                                end
                                            else
                                                count5      <=  count5 + 1'b1;
                                        end
                                    else
                                        count4      <=  count4 + 1'b1;
                                end
                            7   : lpddr2_init_write_task_rw ('h0       ,'hb         ,14 ); // MRW MR10 ZQcal FF
                            14  : //1us delay
                                begin
                                    if (count4 == 100)
                                        begin
                                            lpddr2_init_state   <=  8;
                                            count4            <=  0;
                                        end
                                    else
                                        count4      <=  count4 + 1'b1;
                                end
                            8   : lpddr2_init_write_task_rw ('h0       ,'h3         ,9  ); // MRW MR1
                            9   : lpddr2_init_write_task_rw ('h0       ,'h7         ,10 ); // MRW MR2
                            10  : lpddr2_init_write_task_rw ('h0       ,'h9         ,11 ); // MRW MR3
                            // end of LPDDR2 initialization sequence
                            11  :
                                begin
                                    if (cal_init_req)
                                        begin
                                            lpddr2_init_done      <=  0;
                                            lpddr2_init_state     <=  0;
                                        end
                                    else
                                        lpddr2_init_done    <=  1;
                                end
                        endcase
                end
            
        end
    else
        begin
            assign  lpddr2_init_done        =   0;
            assign  lpddr2_init_address     =   '0;
            assign  lpddr2_init_write       =   0;
            assign  lpddr2_init_writedata   =   '0;
        end
endgenerate
        
    logic [4:0]    write_state;
	logic [AVL_ADDR_WIDTH - 1:0]       write_address;
	logic                              write_write;
	logic [AVL_DATA_WIDTH - 1:0]       write_writedata;
	logic [8:0]    count3;

    always_ff @(posedge clk, negedge reset_n)
        begin
            if (!reset_n)
                begin
                    write_state       <=  0;
                    write_done        <=  0;
                    write_address     <=  '0;
                    write_write       <=  0;
                    write_writedata   <=  '0;
                    count3            <=  '0;
                end
            else
                case(write_state)
                    0   : if (start_write) write_state <=  1;
                    // precharge and activate
                    1   :
                        begin
                            if (&count3)
                                begin
                                    write_state <=  2;
                                    count3      <=  0;
                                end
                            else
                                count3      <=  count3 + 1'b1;
                        end
                    2   : write_write_task_rw ('h0       ,PRE_N_ACT_1 ,3  );
                    3   : write_write_task_rw ('h200     ,PRE_N_ACT_2 ,4  );
                    4   : write_write_task_rw ('h300     ,PRE_N_ACT_3 ,5  );
                    5   : write_write_task_rw ('h201     ,PRE_N_ACT_4 ,6  );
                    6   : write_write_task_rw ('h301     ,PRE_N_ACT_5 ,7  );
                    7   : write_write_task_rw ('h0       ,PRE_N_ACT_6 ,8  );
                    
                    // guaranteed write
                    8   : write_write_task_rw ('h200     ,GRNTEE_WR_1 ,9  );
                    9   : write_write_task_rw ('h300     ,GRNTEE_WR_2 ,10 );
                    10  : write_write_task_rw ('h201     ,GRNTEE_WR_3 ,11 );
                    11  : write_write_task_rw ('h301     ,GRNTEE_WR_4 ,12 );
                    12  : write_write_task_rw ('h202     ,GRNTEE_WR_5 ,13 );
                    13  : write_write_task_rw ('h302     ,GRNTEE_WR_6 ,14 );
                    14  : write_write_task_rw ('h203     ,GRNTEE_WR_7 ,15 );
                    15  : write_write_task_rw ('h303     ,GRNTEE_WR_8 ,16 );
                    16  : write_write_task_rw ('h0       ,GRNTEE_WR_9 ,17 );
                    17  :
                        begin
                            if (cal_init_req)
                                begin
                                    write_done      <=  0;
                                    write_state     <=  0;
                                end
                            else
                                write_done    <=  1;
                        end
                endcase
        end
        
task write_write_task_rw;
    input [AVL_ADDR_WIDTH - 1:0] task_address;
    input [AVL_DATA_WIDTH - 1:0] data;
    input [4:0] next_state;
    
    begin
        write_write       <=  1;
        write_address     <=  RW_MGR_BASE[AVL_ADDR_WIDTH - 1:0] + {task_address[AVL_ADDR_WIDTH - 3:0],2'b0};
        write_writedata   <=  data;
        if (write_write && !waitrequest)
            begin
                write_write       <=  0;
                write_address     <=  '0;
                write_writedata   <=  '0;
                write_state   <=  next_state;
            end
    end
endtask
    
    logic [3:0]    user_mr_state;
	logic [AVL_ADDR_WIDTH - 1:0]       user_mr_address;
	logic                              user_mr_write;
	logic [AVL_DATA_WIDTH - 1:0]       user_mr_writedata;

    always_ff @(posedge clk, negedge reset_n)
        begin
            if (!reset_n)
                begin
                    user_mr_state       <=  0;
                    user_mr_done        <=  0;
                    user_mr_address     <=  '0;
                    user_mr_write       <=  0;
                    user_mr_writedata   <=  '0;
                end
            else
                case(user_mr_state)
                    0   : if (start_user_mr) user_mr_state <=  1;
                    // precharge and activate
                    1   :
                        begin
                            if (HPS_PROTOCOL == "DDR3")
                                user_mr_state   <=  2;
                            else if (HPS_PROTOCOL == "DDR2")
                                user_mr_state   <=  7;
                            else // LPDDR2
                                user_mr_state   <=  11;
                        end
                    
                    // DDR3 user MR start
                    2   : user_mr_write_task_rw ('h0       ,'h14        ,3  );
                    3   : user_mr_write_task_rw ('h0       ,'h7         ,4  );
                    4   : user_mr_write_task_rw ('h0       ,'h9         ,5  );
                    5   : user_mr_write_task_rw ('h0       ,'h5         ,6  );
                    6   : user_mr_write_task_rw ('h0       ,'hd         ,14 );
                    // DDR3 user MR ends
                    
                    // DDR2 user MR start
                    7   : user_mr_write_task_rw ('h0       ,'hd         ,8  );
                    8   : user_mr_write_task_rw ('h0       ,'hf         ,9  );
                    9   : user_mr_write_task_rw ('h0       ,'h9         ,10 );
                    10  : user_mr_write_task_rw ('h0       ,'h3         ,14 );
                    // DDR2 user MR ends
                    
                    // LPDDR2 user MR start
                    11  : user_mr_write_task_rw ('h0       ,'h5         ,12 );
                    12  : user_mr_write_task_rw ('h0       ,'h7         ,13 );
                    13  : user_mr_write_task_rw ('h0       ,'h9         ,14 );
                    // LPDDR2 user MR ends
                    14  :
                        begin
                            if (cal_init_req)
                                begin
                                    user_mr_done      <=  0;
                                    user_mr_state     <=  0;
                                end
                            else
                                user_mr_done    <=  1;
                        end
                endcase
        end
        
task user_mr_write_task_rw;
    input [AVL_ADDR_WIDTH - 1:0] task_address;
    input [AVL_DATA_WIDTH - 1:0] data;
    input [3:0] next_state;
    
    begin
        user_mr_write       <=  1;
        user_mr_address     <=  RW_MGR_BASE[AVL_ADDR_WIDTH - 1:0] + {task_address[AVL_ADDR_WIDTH - 3:0],2'b0};
        user_mr_writedata   <=  data;
        if (user_mr_write && !waitrequest)
            begin
                user_mr_write       <=  0;
                user_mr_address     <=  '0;
                user_mr_writedata   <=  '0;
                user_mr_state   <=  next_state;
            end
    end
endtask
    
    logic [3:0]    pd_state;
	logic [AVL_ADDR_WIDTH - 1:0]       pd_address;
	logic                              pd_write;
	logic                              pd_read;
	logic [AVL_DATA_WIDTH - 1:0]       pd_writedata;
	logic [3:0]    lock_count;
	logic                              pd_down_r;
	logic                              pd_up_r;
	logic [3:0]    cal_chk_count;

    always_ff @(posedge clk, negedge reset_n)
        begin
            if (!reset_n)
                begin
                    pd_down_r      <=  0;
                    pd_up_r        <=  0;
                end
            else
                if (pd_state == 2 || pd_state == 6)
                    begin
                        pd_down_r   <=  pd_down;
                        pd_up_r     <=  pd_up;
                    end
        end
                    
    always_ff @(posedge clk, negedge reset_n)
        begin
            if (!reset_n)
                begin
                    pd_state       <=  0;
                    pd_address     <=  '0;
                    pd_write       <=  0;
                    pd_read        <=  0;
                    pd_writedata   <=  '0;
                    pd_reset_n     <=  0;
                    pd_ack         <=  0;
                    lock_count     <=  '0;
                    cal_init_req   <=  0;
                    cal_chk_count  <=  '0;
                end
            else
                case(pd_state)
                    0   :
                        begin
                            if (pd_start && !cal_init_req)
                                begin
                                    pd_reset_n  <=  1;
                                    pd_state    <=  1;
                                end
                            
                            if (cal_init_req)
                                cal_init_req    <=  0;
                        end
                    1   :
                        begin
                            pd_ack  <=  0;
                            if (pd_down)
                                pd_write_task_tracking_clock (0,2);
                            else if (pd_up)
                                pd_write_task_tracking_clock (1,2);
                        end
                    2   :
                        begin
                            if (pd_down_r && pd_up)
                                lock_count  <=  lock_count + 1'b1;
                            else if (pd_up_r && pd_down)
                                lock_count  <=  lock_count + 1'b1;
                            
                            pd_ack  <=  1;
                            pd_state    <=  3;
                        end
                    3   :
                        begin
                            if (lock_count >= 8)
                                pd_state    <=  4;
                            else
                                pd_state    <=  1;
                        end
                    4   :
                        begin
                            pd_ack  <=  0;
                            if (pd_down)
                                begin
                                    if (pd_down_r || DISABLE_FILTER)
                                        begin
                                            if (DISABLE_PD)
                                                pd_state    <=  5;
                                            else
                                                pd_write_task_capture_clock (0,5);
                                        end
                                    else
                                        pd_state    <=  5;
                                end
                            else if (pd_up)
                                begin
                                    if (pd_up_r || DISABLE_FILTER)
                                        begin
                                            if (DISABLE_PD)
                                                pd_state    <=  5;
                                            else
                                                pd_write_task_capture_clock (1,5);
                                        end
                                    else
                                        pd_state    <=  5;
                                end
                        end
                    5   :
                        begin
                            if (pd_down)
                                pd_write_task_tracking_clock (0,6);
                            else if (pd_up)
                                pd_write_task_tracking_clock (1,6);
                        end
                    6   :
                        begin
                            pd_ack      <=  1;
                            if (cal_chk_count == 4'd10)
                                begin
                                    cal_chk_count   <=  '0;
                                    pd_state    <=  7;
                                end
                            else
                                begin
                                    cal_chk_count   <=  cal_chk_count + 1'b1;
                                    pd_state    <=  8;
                                end
                        end
                    7   :
                        begin
                            pd_read    <=  1;
                            pd_address <=  PLL_MGR_BASE[AVL_ADDR_WIDTH - 1:0]+{2'd2,2'b00}; 
                            if (pd_read && !waitrequest)
                                begin
                                    pd_read       <=  0;
                                    pd_address    <=  '0;
                                    if (readdata[0] == 1)
                                        begin
                                            pd_state        <=  0;
                                            cal_init_req    <=  1;
                                            lock_count      <=  0;
                                        end
                                    else
                                        pd_state      <=  8;
                                end
                        end
                    8   :
                        pd_state    <=  4;
                endcase
        end

task pd_write_task_tracking_clock;
    input data;
    input [2:0] next_state;
    
    begin
        pd_write       <=  1;
        pd_address     <=  PLL_MGR_BASE[AVL_ADDR_WIDTH - 1:0]+{1'b1,2'b00}; // tracking clock
        pd_writedata   <=  data;
        if (pd_write && !waitrequest)
            begin
                pd_write       <=  0;
                pd_address     <=  '0;
                pd_writedata   <=  '0;
                pd_state   <=  next_state;
            end
    end
endtask

task pd_write_task_capture_clock;
    input data;
    input [2:0] next_state;
    
    begin
        pd_write       <=  1;
        pd_address     <=  PLL_MGR_BASE[AVL_ADDR_WIDTH - 1:0];
        pd_writedata   <=  data;
        if (pd_write && !waitrequest)
            begin
                pd_write       <=  0;
                pd_address     <=  '0;
                pd_writedata   <=  '0;
                pd_state   <=  next_state;
            end
    end
endtask

	assign    address        =    main_address | ddr3_init_address | ddr2_init_address | lpddr2_init_address | write_address | user_mr_address | pd_address;
	assign    write          =    main_write | ddr3_init_write | ddr2_init_write | lpddr2_init_write | write_write | user_mr_write | pd_write;
	assign    writedata      =    main_writedata | ddr3_init_writedata | ddr2_init_writedata | lpddr2_init_writedata | write_writedata | user_mr_writedata | pd_writedata;
	assign    read           =    main_read | pd_read;

endmodule
