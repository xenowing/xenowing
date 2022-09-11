use crate::buster::*;
use crate::fifo::*;
use crate::peek_buffer::*;

use kaze::*;

use rtl_meta::bit_pusher::*;
use rtl_meta::xenowing::*;

pub struct BitPusher<'a> {
    pub m: &'a Module<'a>,

    pub reg_port: ReplicaPort<'a>,

    pub sys_port: PrimaryPort<'a>,
    pub mem_port: PrimaryPort<'a>,
}

impl<'a> BitPusher<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> BitPusher<'a> {
        let m = p.module(instance_name, "BitPusher");

        let reg_bus_enable = m.input("reg_bus_enable", 1);
        let reg_bus_addr = m.input("reg_bus_addr", REG_BUS_ADDR_BITS);
        let reg_bus_write = m.input("reg_bus_write", 1);
        let reg_bus_write_data = m.input("reg_bus_write_data", 128);
        let reg_bus_write_byte_enable = m.input("reg_bus_write_byte_enable", 128 / 8);
        let reg_bus_ready = m.output("reg_bus_ready", m.lit(true, 1));
        let reg_bus_read_data_valid = m.output(
            "reg_bus_read_data_valid",
            (reg_bus_enable & !reg_bus_write).reg_next_with_default("reg_bus_read_data_valid_reg", false),
        );

        let reg_write = reg_bus_enable & reg_bus_write;
        let reg_addr = reg_bus_addr.bits(REG_BUS_ADDR_BIT_WIDTH - 1, 0);

        let direction_reg = m.reg("direction_reg", REG_DIRECTION_BITS);
        direction_reg.drive_next(if_(reg_write & reg_addr.eq(m.lit(REG_DIRECTION_ADDR, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(REG_DIRECTION_BITS - 1, 0)
        }).else_({
            direction_reg
        }));

        let start_transfer = reg_write & reg_addr.eq(m.lit(REG_START_ADDR, REG_BUS_ADDR_BIT_WIDTH));
        let write_num_words = reg_write & reg_addr.eq(m.lit(REG_NUM_WORDS_ADDR, REG_BUS_ADDR_BIT_WIDTH));

        let sys_bus_write_byte_enable = m.output("sys_bus_write_byte_enable", m.lit(0xffffu32, 16));
        let sys_bus_ready = m.input("sys_bus_ready", 1);
        let sys_bus_read_data = m.input("sys_bus_read_data", 128);
        let sys_bus_read_data_valid = m.input("sys_bus_read_data_valid", 1);

        let mem_bus_write_byte_enable = m.output("mem_bus_write_byte_enable", m.lit(0xffffu32, 16));
        let mem_bus_ready = m.input("mem_bus_ready", 1);
        let mem_bus_read_data = m.input("mem_bus_read_data", 128);
        let mem_bus_read_data_valid = m.input("mem_bus_read_data_valid", 1);

        let fifo_depth_bits = 5;

        let data_fifo = Fifo::new("data_fifo", fifo_depth_bits, 128, m);
        data_fifo.write_enable.drive(sys_bus_read_data_valid | mem_bus_read_data_valid);
        data_fifo.write_data.drive(if_(sys_bus_read_data_valid, {
            sys_bus_read_data
        }).else_({
            mem_bus_read_data
        }));

        let data_buffer = PeekBuffer::new("data_buffer", 128, m);
        data_fifo.read_enable.drive(data_buffer.ingress_read_enable);
        data_buffer.ingress_data.drive(data_fifo.read_data);
        data_buffer.ingress_data_valid.drive(
            (!data_fifo.empty & data_buffer.ingress_read_enable)
            .reg_next_with_default("data_buffer_ingress_data_valid", false));

        let mem2sys = direction_reg.eq(m.lit(REG_DIRECTION_MEM2SYS, REG_DIRECTION_BITS));

        let read_issue = ReadIssue::new("read_issue", fifo_depth_bits, m);
        read_issue.num_words.drive(reg_bus_write_data.bits(31, 0));
        read_issue.write_num_words.drive(write_num_words);
        read_issue.start_transfer.drive(start_transfer);
        read_issue.bus_ready.drive(if_(mem2sys, {
            mem_bus_ready
        }).else_({
            sys_bus_ready
        }));

        let write_issue = WriteIssue::new("write_issue", m);
        write_issue.num_words.drive(reg_bus_write_data.bits(31, 0));
        write_issue.write_num_words.drive(write_num_words);
        write_issue.start_transfer.drive(start_transfer);
        write_issue.data_ready.drive(data_buffer.egress_ready);
        data_buffer.egress_read_enable.drive(write_issue.issue_accepted);
        write_issue.bus_ready.drive(if_(mem2sys, {
            sys_bus_ready
        }).else_({
            mem_bus_ready
        }));
        read_issue.credit_counter_inc.drive(write_issue.issue_accepted);

        let sys_bus_enable = m.output("sys_bus_enable", if_(mem2sys, {
            write_issue.bus_enable
        }).else_({
            read_issue.bus_enable
        }));
        let sys_bus_write = m.output("sys_bus_write", mem2sys);
        let sys_bus_write_data = m.output("sys_bus_write_data", data_buffer.egress_data);

        let mem_bus_enable = m.output("mem_bus_enable", if_(mem2sys, {
            read_issue.bus_enable
        }).else_({
            write_issue.bus_enable
        }));
        let mem_bus_write = m.output("mem_bus_write", !mem2sys);
        let mem_bus_write_data = m.output("mem_bus_write_data", data_buffer.egress_data);

        let busy = read_issue.busy | write_issue.busy;

        let sys_addr_unit = AddrUnit::new("sys_addr_unit", m);
        sys_addr_unit.write_data.drive(reg_bus_write_data.bits(31, 0));
        sys_addr_unit.write_addr.drive(reg_write & reg_addr.eq(m.lit(REG_SYS_ADDR_ADDR, REG_BUS_ADDR_BIT_WIDTH)));
        sys_addr_unit.write_words_per_span.drive(reg_write & reg_addr.eq(m.lit(REG_SYS_WORDS_PER_SPAN_ADDR, REG_BUS_ADDR_BIT_WIDTH)));
        sys_addr_unit.write_span_stride.drive(reg_write & reg_addr.eq(m.lit(REG_SYS_SPAN_STRIDE_ADDR, REG_BUS_ADDR_BIT_WIDTH)));
        sys_addr_unit.start_transfer.drive(start_transfer);
        sys_addr_unit.step.drive(if_(mem2sys, {
            write_issue.issue_accepted
        }).else_({
            read_issue.issue_accepted
        }));

        let mem_addr_unit = AddrUnit::new("mem_addr_unit", m);
        mem_addr_unit.write_data.drive(reg_bus_write_data.bits(31, 0));
        mem_addr_unit.write_addr.drive(reg_write & reg_addr.eq(m.lit(REG_MEM_ADDR_ADDR, REG_BUS_ADDR_BIT_WIDTH)));
        mem_addr_unit.write_words_per_span.drive(reg_write & reg_addr.eq(m.lit(REG_MEM_WORDS_PER_SPAN_ADDR, REG_BUS_ADDR_BIT_WIDTH)));
        mem_addr_unit.write_span_stride.drive(reg_write & reg_addr.eq(m.lit(REG_MEM_SPAN_STRIDE_ADDR, REG_BUS_ADDR_BIT_WIDTH)));
        mem_addr_unit.start_transfer.drive(start_transfer);
        mem_addr_unit.step.drive(if_(mem2sys, {
            read_issue.issue_accepted
        }).else_({
            write_issue.issue_accepted
        }));

        let sys_bus_addr = m.output("sys_bus_addr", sys_addr_unit.addr);

        let mem_bus_addr = m.output("mem_bus_addr", mem_addr_unit.addr);

        let reg_bus_read_data = m.output("reg_bus_read_data", m.lit(0u32, 127).concat(busy));

        BitPusher {
            m,

            reg_port: ReplicaPort {
                bus_enable: reg_bus_enable,
                bus_addr: reg_bus_addr,
                bus_write: reg_bus_write,
                bus_write_data: reg_bus_write_data,
                bus_write_byte_enable: reg_bus_write_byte_enable,
                bus_ready: reg_bus_ready,
                bus_read_data: reg_bus_read_data,
                bus_read_data_valid: reg_bus_read_data_valid,
            },

            sys_port: PrimaryPort {
                bus_enable: sys_bus_enable,
                bus_addr: sys_bus_addr,
                bus_write: sys_bus_write,
                bus_write_data: sys_bus_write_data,
                bus_write_byte_enable: sys_bus_write_byte_enable,
                bus_ready: sys_bus_ready,
                bus_read_data: sys_bus_read_data,
                bus_read_data_valid: sys_bus_read_data_valid,
            },
            mem_port: PrimaryPort {
                bus_enable: mem_bus_enable,
                bus_addr: mem_bus_addr,
                bus_write: mem_bus_write,
                bus_write_data: mem_bus_write_data,
                bus_write_byte_enable: mem_bus_write_byte_enable,
                bus_ready: mem_bus_ready,
                bus_read_data: mem_bus_read_data,
                bus_read_data_valid: mem_bus_read_data_valid,
            },
        }
    }
}

struct ReadIssue<'a> {
    busy: &'a Output<'a>,

    num_words: &'a Input<'a>,
    write_num_words: &'a Input<'a>,
    start_transfer: &'a Input<'a>,

    bus_ready: &'a Input<'a>,
    bus_enable: &'a Output<'a>,
    issue_accepted: &'a Output<'a>,
    credit_counter_inc: &'a Input<'a>,
}

impl<'a> ReadIssue<'a> {
    pub fn new(instance_name: impl Into<String>, fifo_depth_bits: u32, p: &'a impl ModuleParent<'a>) -> ReadIssue<'a> {
        let m = p.module(instance_name, "ReadIssue");

        let busy_reg = m.reg("busy_reg", 1);
        busy_reg.default_value(false);

        let busy = m.output("busy", busy_reg);

        let num_words = m.input("num_words", 32);
        let write_num_words = m.input("write_num_words", 1);
        let start_transfer = m.input("start_transfer", 1);

        let num_words_reg = m.reg("num_words_reg", 32);

        let bus_ready = m.input("bus_ready", 1);
        let credit_counter_inc = m.input("credit_counter_inc", 1);

        let credit_counter_bits = fifo_depth_bits + 1;
        let credit_counter = m.reg("credit_counter", credit_counter_bits);
        credit_counter.default_value(1u32 << fifo_depth_bits);

        let issue = busy_reg & credit_counter.ne(m.lit(0u32, credit_counter_bits));
        let issue_accepted = issue & bus_ready;

        let decremented_num_words = num_words_reg - m.lit(1u32, 32);

        busy_reg.drive_next(if_(start_transfer, {
            m.lit(true, 1)
        }).else_if(issue_accepted & decremented_num_words.eq(m.lit(0u32, 32)), {
            m.lit(false, 1)
        }).else_({
            busy_reg
        }));

        num_words_reg.drive_next(if_(write_num_words, {
            num_words as &dyn Signal<'a>
        }).else_if(issue_accepted, {
            decremented_num_words
        }).else_({
            num_words_reg
        }));

        credit_counter.drive_next(if_(!issue_accepted & credit_counter_inc, {
            credit_counter + m.lit(1u32, credit_counter_bits)
        }).else_if(issue_accepted & !credit_counter_inc, {
            credit_counter - m.lit(1u32, credit_counter_bits)
        }).else_({
            credit_counter
        }));

        let issue_accepted = m.output("issue_accepted", issue_accepted);
        let bus_enable = m.output("bus_enable", issue);

        ReadIssue {
            busy,

            num_words,
            write_num_words,
            start_transfer,

            bus_ready,
            bus_enable,
            issue_accepted,
            credit_counter_inc,
        }
    }
}

struct WriteIssue<'a> {
    busy: &'a Output<'a>,

    num_words: &'a Input<'a>,
    write_num_words: &'a Input<'a>,
    start_transfer: &'a Input<'a>,

    data_ready: &'a Input<'a>,
    bus_ready: &'a Input<'a>,
    bus_enable: &'a Output<'a>,
    issue_accepted: &'a Output<'a>,
}

impl<'a> WriteIssue<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> WriteIssue<'a> {
        let m = p.module(instance_name, "WriteIssue");

        let busy_reg = m.reg("busy_reg", 1);
        busy_reg.default_value(false);

        let busy = m.output("busy", busy_reg);

        let num_words = m.input("num_words", 32);
        let write_num_words = m.input("write_num_words", 1);
        let start_transfer = m.input("start_transfer", 1);

        let num_words_reg = m.reg("num_words_reg", 32);

        let data_ready = m.input("data_ready", 1);
        let bus_ready = m.input("bus_ready", 1);

        let issue = busy_reg & data_ready;
        let issue_accepted = issue & bus_ready;

        let decremented_num_words = num_words_reg - m.lit(1u32, 32);

        busy_reg.drive_next(if_(start_transfer, {
            m.lit(true, 1)
        }).else_if(issue_accepted & decremented_num_words.eq(m.lit(0u32, 32)), {
            m.lit(false, 1)
        }).else_({
            busy_reg
        }));

        num_words_reg.drive_next(if_(write_num_words, {
            num_words as &dyn Signal<'a>
        }).else_if(issue_accepted, {
            decremented_num_words
        }).else_({
            num_words_reg
        }));

        let bus_enable = m.output("bus_enable", issue);
        let issue_accepted = m.output("issue_accepted", issue_accepted);

        WriteIssue {
            busy,

            num_words,
            write_num_words,
            start_transfer,

            data_ready,
            bus_ready,
            bus_enable,
            issue_accepted,
        }
    }
}

struct AddrUnit<'a> {
    write_data: &'a Input<'a>,
    write_addr: &'a Input<'a>,
    write_words_per_span: &'a Input<'a>,
    write_span_stride: &'a Input<'a>,

    start_transfer: &'a Input<'a>,
    step: &'a Input<'a>,

    addr: &'a Output<'a>,
}

impl<'a> AddrUnit<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> AddrUnit<'a> {
        let m = p.module(instance_name, "AddrUnit");

        let write_data = m.input("write_data", 32);
        let write_addr = m.input("write_addr", 1);
        let write_words_per_span = m.input("write_words_per_span", 1);
        let write_span_stride = m.input("write_span_stride", 1);

        let start_transfer = m.input("start_transfer", 1);
        let step = m.input("step", 1);

        let addr_reg = m.reg("addr_reg", SYSTEM_BUS_ADDR_BITS);
        let words_per_span_reg = m.reg("words_per_span_reg", 32);
        let span_stride_reg = m.reg("span_stride_reg", SYSTEM_BUS_ADDR_BITS);

        let span_base_reg = m.reg("span_base_reg", SYSTEM_BUS_ADDR_BITS);
        let span_word_counter_reg = m.reg("span_word_counter_reg", 32);

        let next_addr = addr_reg + m.lit(1u32, SYSTEM_BUS_ADDR_BITS);
        let next_span_word_counter = span_word_counter_reg + m.lit(1u32, 32);
        let next_span = next_span_word_counter.eq(words_per_span_reg);
        let next_span_base = span_base_reg + span_stride_reg;

        addr_reg.drive_next(if_(write_addr, {
            write_data.bits(SYSTEM_BUS_ADDR_BITS + 4 - 1, 4)
        }).else_if(step, {
            if_(next_span, {
                next_span_base
            }).else_({
                next_addr
            })
        }).else_({
            addr_reg
        }));

        words_per_span_reg.drive_next(if_(write_words_per_span, {
            write_data
        }).else_({
            words_per_span_reg
        }));

        span_stride_reg.drive_next(if_(write_span_stride, {
            write_data.bits(SYSTEM_BUS_ADDR_BITS - 1, 0)
        }).else_({
            span_stride_reg
        }));

        span_base_reg.drive_next(if_(step & next_span, {
            next_span_base
        }).else_if(start_transfer, {
            addr_reg
        }).else_({
            span_base_reg
        }));

        span_word_counter_reg.drive_next(if_(step, {
            if_(next_span, {
                m.lit(0u32, 32)
            }).else_({
                next_span_word_counter
            })
        }).else_if(start_transfer, {
            m.lit(0u32, 32)
        }).else_({
            span_word_counter_reg
        }));

        let addr = m.output("addr", addr_reg);

        AddrUnit {
            write_data,
            write_addr,
            write_words_per_span,
            write_span_stride,

            start_transfer,
            step,

            addr,
        }
    }
}
