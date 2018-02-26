
#include "sequencer_defines.h"

#include "alt_types.h"
#include "system.h"
#include "io.h"
#include "sequencer_m10.h"
#include "sequencer_auto.h"

// Just to make the debugging code more uniform
#ifndef RW_MGR_MEM_NUMBER_OF_CS_PER_DIMM
#define RW_MGR_MEM_NUMBER_OF_CS_PER_DIMM 0
#endif

// To make CALIB_SKIP_DELAY_LOOPS a dynamic conditional option
// instead of static, we use boolean logic to select between
// non-skip and skip values
//
// The mask is set to include all bits when not-skipping, but is
// zero when skipping

gbl_t *gbl = 0;
param_t *param = 0;

void initialize(void)
{
	alt_u32 i;
	
	// Initial LFIFO is max
	
	IOWR_32DIRECT (PHY_MGR_PHY_RLAT, 0, 0xf);

	//USER calibration has control over path to memory 

	IOWR_32DIRECT (PHY_MGR_MUX_SEL, 0, 1);

	//USER memory clock is not stable we begin initialization 

	IOWR_32DIRECT (PHY_MGR_RESET_MEM_STBL, 0, 0);

	//USER calibration status all set to zero 

	IOWR_32DIRECT (PHY_MGR_CAL_STATUS, 0, 0);

	param->read_correct_mask_vg  = ((t_btfld)1 << (RW_MGR_MEM_DQ_PER_READ_DQS / RW_MGR_MEM_VIRTUAL_GROUPS_PER_READ_DQS)) - 1;
	param->write_correct_mask_vg = ((t_btfld)1 << (RW_MGR_MEM_DQ_PER_READ_DQS / RW_MGR_MEM_VIRTUAL_GROUPS_PER_READ_DQS)) - 1;
	param->read_correct_mask     = ((t_btfld)1 << RW_MGR_MEM_DQ_PER_READ_DQS) - 1;
	param->write_correct_mask    = ((t_btfld)1 << RW_MGR_MEM_DQ_PER_WRITE_DQS) - 1;
	param->dm_correct_mask       = ((t_btfld)1 << (RW_MGR_MEM_DATA_WIDTH / RW_MGR_MEM_DATA_MASK_WIDTH)) - 1;
}

void set_rank_and_odt_mask(alt_u32 rank)
{
	alt_u32 odt_mask_0 = 0;
	alt_u32 odt_mask_1 = 0;
	alt_u32 cs_and_odt_mask;
	
	odt_mask_0 = 0x0;
	odt_mask_1 = 0x0;

	cs_and_odt_mask = 
		(0xFF & ~(1 << rank)) |
		((0xFF & odt_mask_0) << 8) |
		((0xFF & odt_mask_1) << 16);

	IOWR_32DIRECT (RW_MGR_SET_CS_AND_ODT_MASK, 0, cs_and_odt_mask);
}

// should always use constants as argument to ensure all computations are performed at compile time
static inline void delay_for_n_mem_clocks(const alt_u32 clocks)
{
	alt_u32 afi_clocks;
	alt_u8 inner;
	alt_u8 outer;
	alt_u16 c_loop;

	afi_clocks = (clocks + AFI_RATE_RATIO-1) / AFI_RATE_RATIO; /* scale (rounding up) to get afi clocks */

	// Note, we don't bother accounting for being off a little bit because of a few extra instructions in outer loops
	// Note, the loops have a test at the end, and do the test before the decrement, and so always perform the loop
	// 1 time more than the counter value
	if (afi_clocks == 0) {
		inner = outer = c_loop = 0;
	} else if (afi_clocks <= 0x100) {
		inner = afi_clocks-1;
		outer = 0;
		c_loop = 0;
	} else if (afi_clocks <= 0x10000) {
		inner = 0xff;
		outer = (afi_clocks-1) >> 8;
		c_loop = 0;
	} else {
		inner = 0xff;
		outer = 0xff;
		c_loop = (afi_clocks-1) >> 16;
	}

	// rom instructions are structured as follows:
	//
	//    IDLE_LOOP2: jnz cntr0, TARGET_A
	//    IDLE_LOOP1: jnz cntr1, TARGET_B
	//                return
	//
	// so, when doing nested loops, TARGET_A is set to IDLE_LOOP2, and TARGET_B is
	// set to IDLE_LOOP2 as well
	//
	// if we have no outer loop, though, then we can use IDLE_LOOP1 only, and set
	// TARGET_B to IDLE_LOOP1 and we skip IDLE_LOOP2 entirely
	//
	// a little confusing, but it helps save precious space in the inst_rom and sequencer rom
	// and keeps the delays more accurate and reduces overhead
	if (afi_clocks <= 0x100) {

		IOWR_32DIRECT (RW_MGR_LOAD_CNTR_1, 0, inner);
		IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_1, 0, __RW_MGR_IDLE_LOOP1);
		IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_IDLE_LOOP1);

	} else {
		IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, inner);
		IOWR_32DIRECT (RW_MGR_LOAD_CNTR_1, 0, outer);
	
		IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_0, 0, __RW_MGR_IDLE_LOOP2);
		IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_1, 0, __RW_MGR_IDLE_LOOP2);

		// hack to get around compiler not being smart enough
		if (afi_clocks <= 0x10000) {
			// only need to run once
			IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_IDLE_LOOP2);
		} else {
			do {
				IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_IDLE_LOOP2);
			} while (c_loop-- != 0);
		}
	}

}

// should always use constants as argument to ensure all computations are performed at compile time
static inline void delay_for_n_ns(const alt_u32 nanoseconds)
{
	delay_for_n_mem_clocks((1000*nanoseconds) / (1000000/AFI_CLK_FREQ) * AFI_RATE_RATIO);
}

#if DDR3
void rw_mgr_mem_initialize (void)
{
	//USER The reset / cke part of initialization is broadcasted to all ranks
	IOWR_32DIRECT (RW_MGR_SET_CS_AND_ODT_MASK, 0, RW_MGR_RANK_ALL);

	//USER start with memory RESET activated
	//USER tINIT = 200us

	//USER Load counters
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, 0x2);//was 0xFF
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_1, 0, 0x2);//was 0x6A
	
	//USER Load jump address
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_0, 0, __RW_MGR_INIT_RESET_0_CKE_0);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_1, 0, __RW_MGR_INIT_RESET_0_CKE_0_inloop);

	//USER Execute count instruction
	//USER IOWR_32DIRECT (BASE_RW_MGR, 0, __RW_MGR_COUNT_REG_0);
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_INIT_RESET_0_CKE_0);

	//USER indicate that memory is stable
	IOWR_32DIRECT (PHY_MGR_RESET_MEM_STBL, 0, 1);

	//USER transition the RESET to high 
	//USER Wait for 500us

	//USER Load counters
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, 0x2);//was 0x83
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_1, 0, 0x2);//was 0xFF

	//USER Load jump address
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_0, 0, __RW_MGR_INIT_RESET_1_CKE_0);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_1, 0, __RW_MGR_INIT_RESET_1_CKE_0_inloop_1);

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_INIT_RESET_1_CKE_0);

	//USER bring up clock enable 

	//USER tXRP < 250 ck cycles
	delay_for_n_mem_clocks(250);

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MRS2);
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MRS3);
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MRS1);
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MRS0_DLL_RESET);
    
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_ZQCL);
    
	//USER tZQinit = tDLLK = 512 ck cycles
	delay_for_n_mem_clocks(512);

}
#endif // DDR3

#if DDR2
void rw_mgr_mem_initialize (void)
{
	//USER start with CKE low 
	//USER tINIT = 200us

	//USER Load counters
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, 0x2);//was 0xFF
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_1, 0, 0x2);//was 0x76
	
	//USER Load jump address
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_0, 0, __RW_MGR_INIT_CKE_0);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_1, 0, __RW_MGR_INIT_CKE_0_inloop);

	//USER Execute count instruction
	//USER IOWR_32DIRECT (BASE_RW_MGR, 0, __RW_MGR_COUNT_REG_0);
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_INIT_CKE_0);

	//USER indicate that memory is stable 
	IOWR_32DIRECT (PHY_MGR_RESET_MEM_STBL, 0, 1);

	//USER Bring up CKE 
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_NOP);

	//USER *** STAGE (4)

	//USER Wait for 400ns 
	delay_for_n_ns(400);

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_PRECHARGE_ALL);
    
	//USER *** STAGE (5)
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_EMR2);
    
	//USER *** STAGE (6)
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_EMR3);
    
	//USER *** STAGE (7)
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_EMR);
    
	//USER *** STAGE (8)
	//USER DLL reset
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR_DLL_RESET);
    
	//USER *** STAGE (9)
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_PRECHARGE_ALL);
    
	//USER *** STAGE (10)
    
	//USER Issue 2 refresh commands spaced by tREF 
    
	//USER First REFRESH
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_REFRESH);
    
	//USER tREF = 200ns
	delay_for_n_ns(200);
    
	//USER Second REFRESH
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_REFRESH);
    
	//USER Second idle loop
	delay_for_n_ns(200);
    
	//USER *** STAGE (11)
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR_CALIB);
    
	//USER *** STAGE (12)
	//USER OCD defaults
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_EMR_OCD_ENABLE);
    
	//USER *** STAGE (13)
	//USER OCD exit
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_EMR);
    
	//USER *** STAGE (14)
    
	//USER The memory is now initialized. Before being able to use it, we must still
	//USER wait for the DLL to lock, 200 clock cycles after it was reset @ STAGE (8).
	//USER Since we cannot keep track of time in any other way, let's start counting from now
	delay_for_n_mem_clocks(200);
}
#endif // DDR2 

#if LPDDR2
void rw_mgr_mem_initialize (void)
{
	//USER start with CKE low 
	//USER tINIT1 = 100ns

	//USER Load counter
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, 0x10);
	
	//USER Load jump address
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_0, 0, __RW_MGR_INIT_CKE_0);

	//USER Execute count instruction
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_INIT_CKE_0);

	//USER tINIT3 = 200us
	delay_for_n_ns(200000);

	//USER indicate that memory is stable 
	IOWR_32DIRECT (PHY_MGR_RESET_MEM_STBL, 0, 1);

	//USER MRW RESET
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR63_RESET);

	//USER tINIT5 = 10us
	delay_for_n_ns(10000);

	//USER MRW ZQC
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR10_ZQC);
    
	//USER tZQINIT = 1us
	delay_for_n_ns(1000);
    
	//USER MRW MR1
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR1_CALIB);
    
	//USER MRW MR2
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR2);
    
	//USER MRW MR3
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR3);
}
#endif // LPDDR2 

//USER  At the end of calibration we have to program the user settings in, and
//USER  hand off the memory to the user.

#if DDR3
void rw_mgr_mem_handoff (void)
{
	//USER precharge all banks ... 
    
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_PRECHARGE_ALL);
    
	//USER load up MR settings specified by user 
    
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MRS2);
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MRS3);
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MRS1);
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MRS0_USER);
}
#endif // DDR3

#if DDR2
void rw_mgr_mem_handoff (void)
{
	//USER precharge all banks ... 

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_PRECHARGE_ALL);

	//USER load up MR settings specified by user 

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_EMR2);

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_EMR3);

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_EMR);

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR_USER);
}
#endif //USER DDR2

#if LPDDR2
void rw_mgr_mem_handoff (void)
{
	//USER precharge all banks...

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_PRECHARGE_ALL);

	//USER load up MR settings specified by user

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR1_USER);

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR2);

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_MR3);
}
#endif //USER LPDDR2

//USER load up the patterns we are going to use during a read test 
void rw_mgr_mem_calibrate_read_load_patterns ()
{
	//USER Load up a constant bursts
    
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, 0x20);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_0, 0, __RW_MGR_GUARANTEED_WRITE_WAIT0);
    
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_1, 0, 0x20);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_1, 0, __RW_MGR_GUARANTEED_WRITE_WAIT1);
    
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_2, 0, 0x02);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_2, 0, __RW_MGR_GUARANTEED_WRITE_WAIT2);
    
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_3, 0, 0x02);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_3, 0, __RW_MGR_GUARANTEED_WRITE_WAIT3);
    
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_GUARANTEED_WRITE);
}

//USER  try a read and see if it returns correct data back. has dummy reads inserted into the mix
//USER  used to align dqs enable. has more thorough checks than the regular read test.
alt_u32 rw_mgr_mem_calibrate_read_test (alt_u32 group, alt_u32 num_tries, alt_u32 all_correct, t_btfld *bit_chk, alt_u32 all_groups)
{
	alt_u32 vg;
	t_btfld correct_mask_vg;
	t_btfld tmp_bit_chk;

	*bit_chk = param->read_correct_mask;
	correct_mask_vg = param->read_correct_mask_vg;
	
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_1, 0, 0x10);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_1, 0, __RW_MGR_READ_B2B_WAIT1);
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_2, 0, 0x10);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_2, 0, __RW_MGR_READ_B2B_WAIT2);
	
	if (all_groups) {
		IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, 0x06);
	} else {
		IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, 0x32);
	}
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_0, 0, __RW_MGR_READ_B2B);
	if(all_groups) {
		IOWR_32DIRECT (RW_MGR_LOAD_CNTR_3, 0, RW_MGR_MEM_IF_READ_DQS_WIDTH * RW_MGR_MEM_VIRTUAL_GROUPS_PER_READ_DQS - 1);
	} else {
		IOWR_32DIRECT (RW_MGR_LOAD_CNTR_3, 0, 0x0);
	}
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_3, 0, __RW_MGR_READ_B2B);
	
	tmp_bit_chk = 0;
	for (vg = RW_MGR_MEM_VIRTUAL_GROUPS_PER_READ_DQS-1; ; vg--)
	{
		//USER reset the fifos to get pointers to known state 

		IOWR_32DIRECT (PHY_MGR_CMD_FIFO_RESET, 0, 0);
		IOWR_32DIRECT (RW_MGR_RESET_READ_DATAPATH, 0, 0);	

		tmp_bit_chk = tmp_bit_chk << (RW_MGR_MEM_DQ_PER_READ_DQS / RW_MGR_MEM_VIRTUAL_GROUPS_PER_READ_DQS);

		IOWR_32DIRECT (all_groups ? RW_MGR_RUN_ALL_GROUPS : RW_MGR_RUN_SINGLE_GROUP, ((group*RW_MGR_MEM_VIRTUAL_GROUPS_PER_READ_DQS+vg) << 2), __RW_MGR_READ_B2B);
		tmp_bit_chk = tmp_bit_chk | (correct_mask_vg & ~(IORD_32DIRECT(BASE_RW_MGR, 0)));

		if (vg == 0) {
			break;
		}
	}
	*bit_chk &= tmp_bit_chk;

	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, (group << 2), __RW_MGR_CLEAR_DQS_ENABLE);
	
	if (all_correct)
	{
		return (*bit_chk == param->read_correct_mask);
	}
	else
	{
		return (*bit_chk != 0x00);
	}
}

//USER find a good dqs enable to use 
alt_u32 rw_mgr_mem_calibrate_vfifo_find_dqs_en_phase (alt_u32 grp)
{
	alt_u32 v;
	alt_u32 p;
	t_btfld bit_chk;
	alt_u32 found_vfifo;
	alt_u32 vfifo_not_found;
	alt_u32 counter;
	alt_u32 test_status;
	alt_u32 fail_to_fail_phase;
	alt_u32 capture_clk;
	
	if (grp == 2) {
	    capture_clk = 1;
	} else {
	    capture_clk = grp;
	}

	//USER ********************************************************
	//USER * step 2: find working VFIFO
	found_vfifo = 0;
	vfifo_not_found = 0;
	counter = 0;
	while (1) {
	    for (v = 0; v < RTL_VFIFO_SIZE; v++) {
		    test_status = rw_mgr_mem_calibrate_read_test (grp, 1, PASS_ALL_BITS, &bit_chk, 0);
		    
		    if (test_status) {
				found_vfifo = 1;
				break;
			}
			
			IOWR_32DIRECT (PHY_MGR_CMD_INC_VFIFO_HR, 0, grp);
		}
		
		if (found_vfifo) {
			break;
		}
		
		PLL_INCR(capture_clk << 2);
		
		#if LPDDR2
		PLL_INCR(capture_clk+0x4);
		#endif
		
		// 600 steps of 10ps is one full clock at 167Mhz
		// 10ps is the PLL phase step resolution
		// trying this value for now, we know that sometimes we need to sweep two clock due to reset condition of div 2 register
		counter++;
		if (counter == 600) {
		    vfifo_not_found = 1;
		    break;
		}
	}
	
	//USER ********************************************************
	//USER * step 3: decrement phase until read fails
	
	while (1) {
	    if (vfifo_not_found) {
	        break;
	    }
	    
	    // decrement first because the current phase would work
	    PLL_DECR(capture_clk << 2);
	    
	    #if LPDDR2
        PLL_DECR(capture_clk+0x4);
        #endif
	    
	    test_status = rw_mgr_mem_calibrate_read_test (grp, 1, PASS_ONE_BIT, &bit_chk, 0);
	    
	    if (!test_status) {
	        break;
	    }
	}
	
	//USER ********************************************************
	//USER * step 4: increment phase until read fails to determine window
	fail_to_fail_phase = 0;
	while (1) {
	    if (vfifo_not_found) {
	        break;
	    }
	    
	    // increment first since the current phase wouldnt work
	    PLL_INCR(capture_clk << 2);
	    
	    #if LPDDR2
        PLL_INCR(capture_clk+0x4);
        #endif
	    
	    fail_to_fail_phase++;
	    
	    test_status = rw_mgr_mem_calibrate_read_test (grp, 1, PASS_ONE_BIT, &bit_chk, 0);
	    
	    if (!test_status) {
	        break;
	    }
	}
	
	//back by half to get best margin
	for (p = 0; p < (fail_to_fail_phase/2); p++) {
	    PLL_DECR(capture_clk << 2);
	    
	    #if LPDDR2
        PLL_DECR(capture_clk+0x4);
        #endif
	}
	
	if (found_vfifo) {
		return 1;
	} else {
	    return 0;
	}
}

//USER VFIFO Calibration -- Full Calibration
alt_u32 rw_mgr_mem_calibrate_vfifo ()
{
	rw_mgr_mem_calibrate_read_load_patterns ();
	
	int group0=0;
	int group1=0;
	
	#if M10_DQ_WIDTH_8
	group0=rw_mgr_mem_calibrate_vfifo_find_dqs_en_phase(0);
	group1=1;
	#endif
	
	#if M10_DQ_WIDTH_16
	    #if TW0_CAPTURE_CLOCKS
	group0=rw_mgr_mem_calibrate_vfifo_find_dqs_en_phase(0);
	group1=rw_mgr_mem_calibrate_vfifo_find_dqs_en_phase(1);
	    #else
	group0=rw_mgr_mem_calibrate_vfifo_find_dqs_en_phase(0);
	group1=1;
	    #endif
	#endif
	
	#if M10_DQ_WIDTH_24
	group0=rw_mgr_mem_calibrate_vfifo_find_dqs_en_phase(0);
	group1=rw_mgr_mem_calibrate_vfifo_find_dqs_en_phase(2);
	#endif
	
	if (group0 && group1) {
		return 1;
	} else {
	    return 0;
	}
}

//USER precharge all banks and activate row 0 in bank "000..." and bank "111..." 
void mem_precharge_and_activate (void)
{
	//USER precharge all banks ... 
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_PRECHARGE_ALL);
    
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_0, 0, 0x0F);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_0, 0, __RW_MGR_ACTIVATE_0_AND_1_WAIT1);
    
	IOWR_32DIRECT (RW_MGR_LOAD_CNTR_1, 0, 0x0F);
	IOWR_32DIRECT (RW_MGR_LOAD_JUMP_ADD_1, 0, __RW_MGR_ACTIVATE_0_AND_1_WAIT2);
    
	//USER activate rows 
	IOWR_32DIRECT (RW_MGR_RUN_SINGLE_GROUP, 0, __RW_MGR_ACTIVATE_0_AND_1);
}

//USER Configure various memory related parameters.
void mem_config (void)
{
	alt_u32 wlat;

	//USER read in write and read latency 

	wlat = IORD_32DIRECT (MEM_T_WL_ADD, 0);
	
	//USER write latency 
	wlat = (wlat - 1) / 2 + 1;

	//USER advertise write latency 
	gbl->curr_write_lat = wlat;
	IOWR_32DIRECT (PHY_MGR_AFI_WLAT, 0, wlat - 1);
	
	mem_precharge_and_activate ();
}

//USER Memory calibration entry point
 
alt_u32 mem_calibrate (void)
{
	// Initialize the data settings

	gbl->error_substage = CAL_SUBSTAGE_NIL;
	gbl->error_stage = CAL_STAGE_NIL;
	gbl->error_group = 0xff;

	mem_config ();
	
	//USER Calibrate the VFIFO
	
	if (!rw_mgr_mem_calibrate_vfifo ()) {
	    return 0;
	} else {
	    return 1;
	}
}

alt_u32 run_mem_calibrate(void) {
	alt_u32 pass;

    initialize();
	
    set_rank_and_odt_mask(0);
    
	rw_mgr_mem_initialize ();

	pass = mem_calibrate ();

	mem_precharge_and_activate ();
	
	IOWR_32DIRECT (PHY_MGR_CMD_FIFO_RESET, 0, 0);

	//USER Handoff

	rw_mgr_mem_handoff ();

	IOWR_32DIRECT (PHY_MGR_MUX_SEL, 0, 0);

	if (pass) {

		IOWR_32DIRECT (PHY_MGR_CAL_STATUS, 0, PHY_MGR_CAL_SUCCESS);

	} else {
		
		IOWR_32DIRECT (PHY_MGR_CAL_STATUS, 0, PHY_MGR_CAL_FAIL);
	}

	return pass;
}

int main(void)
{
	param_t my_param;
	gbl_t my_gbl;
	alt_u32 pass;

	param = &my_param;
	gbl = &my_gbl;

	IOWR_32DIRECT (RW_MGR_SOFT_RESET, 0, 0);

	pass = run_mem_calibrate ();

	while (1) {
	}

	return pass;
}
