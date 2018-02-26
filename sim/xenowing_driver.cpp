#include "../obj_dir/Vxenowing.h"
#include "verilated_vcd_c.h"

#include <cinttypes>
#include <iostream>
using namespace std;

#ifdef _WIN32
#include <Windows.h>
#else
#include <dlfcn.h>
#endif

typedef struct
{
    uint32_t (*get_reset_n)();
    void (*set_reset_n)(uint32_t value);
    uint32_t (*get_clk)();
    void (*set_clk)(uint32_t value);

    uint32_t (*get_program_rom_addr)();
    void (*set_program_rom_addr)(uint32_t value);
    uint32_t (*get_program_rom_q)();
    void (*set_program_rom_q)(uint32_t value);

    uint32_t (*get_leds)();
    void (*set_leds)(uint32_t value);

    uint32_t (*get_avl_ready)();
    void (*set_avl_ready)(uint32_t value);
    uint32_t (*get_avl_burstbegin)();
    void (*set_avl_burstbegin)(uint32_t value);
    uint32_t (*get_avl_addr)();
    void (*set_avl_addr)(uint32_t value);
    uint32_t (*get_avl_rdata_valid)();
    void (*set_avl_rdata_valid)(uint32_t value);
    uint64_t (*get_avl_rdata)();
    void (*set_avl_rdata)(uint64_t value);
    uint64_t (*get_avl_wdata)();
    void (*set_avl_wdata)(uint64_t value);
    uint32_t (*get_avl_be)();
    void (*set_avl_be)(uint32_t value);
    uint32_t (*get_avl_read_req)();
    void (*set_avl_read_req)(uint32_t value);
    uint32_t (*get_avl_write_req)();
    void (*set_avl_write_req)(uint32_t value);
    uint32_t (*get_avl_size)();
    void (*set_avl_size)(uint32_t value);

    void (*eval)();
    void (*final)();
    void (*trace_dump)(uint64_t time);
} Env;

static Vxenowing *top = nullptr;
static VerilatedVcdC *trace = nullptr;

uint32_t get_reset_n()
{
    return top->reset_n;
}

void set_reset_n(uint32_t value)
{
    top->reset_n = value;
}

uint32_t get_clk()
{
    return top->clk;
}

void set_clk(uint32_t value)
{
    top->clk = value;
}

uint32_t get_program_rom_addr()
{
    return top->program_rom_addr;
}

void set_program_rom_addr(uint32_t value)
{
    top->program_rom_addr = value;
}

uint32_t get_program_rom_q()
{
    return top->program_rom_q;
}

void set_program_rom_q(uint32_t value)
{
    top->program_rom_q = value;
}

uint32_t get_leds()
{
    return top->leds;
}

void set_leds(uint32_t value)
{
    top->leds = value;
}

uint32_t get_avl_ready()
{
    return top->avl_ready;
}

void set_avl_ready(uint32_t value)
{
    top->avl_ready = value;
}

uint32_t get_avl_burstbegin()
{
    return top->avl_burstbegin;
}

void set_avl_burstbegin(uint32_t value)
{
    top->avl_burstbegin = value;
}

uint32_t get_avl_addr()
{
    return top->avl_addr;
}

void set_avl_addr(uint32_t value)
{
    top->avl_addr = value;
}

uint32_t get_avl_rdata_valid()
{
    return top->avl_rdata_valid;
}

void set_avl_rdata_valid(uint32_t value)
{
    top->avl_rdata_valid = value;
}

uint64_t get_avl_rdata()
{
    return top->avl_rdata;
}

void set_avl_rdata(uint64_t value)
{
    top->avl_rdata = value;
}

uint64_t get_avl_wdata()
{
    return top->avl_wdata;
}

void set_avl_wdata(uint64_t value)
{
    top->avl_wdata = value;
}

uint32_t get_avl_be()
{
    return top->avl_be;
}

void set_avl_be(uint32_t value)
{
    top->avl_be = value;
}

uint32_t get_avl_read_req()
{
    return top->avl_read_req;
}

void set_avl_read_req(uint32_t value)
{
    top->avl_read_req = value;
}

uint32_t get_avl_write_req()
{
    return top->avl_write_req;
}

void set_avl_write_req(uint32_t value)
{
    top->avl_write_req = value;
}

uint32_t get_avl_size()
{
    return top->avl_size;
}

void set_avl_size(uint32_t value)
{
    top->avl_size = value;
}

void eval()
{
    top->eval();
}

void final()
{
    top->final();
}

void trace_dump(uint64_t time)
{
    if (trace)
        trace->dump((vluint64_t)time);
}

int main(int argc, char **argv)
{
    if (argc != 2 && argc != 3)
    {
        cout << "Invalid number of arguments" << endl;
        exit(1);
    }

    auto libName = argv[1];
    auto lib =
#ifdef _WIN32
        LoadLibrary(libName);
#else
        dlopen(libName, RTLD_LAZY);
#endif
    if (!lib)
    {
        cout << "Failed to load library: " << libName << endl;
        exit(1);
    }
    auto run = (int32_t (*)(Env *))
#ifdef _WIN32
        GetProcAddress
#else
        dlsym
#endif
        (lib, "run");
    if (!run)
    {
        cout << "Failed to get run proc address" << endl;
        exit(1);
    }

    top = new Vxenowing();

    if (argc == 3)
    {
        Verilated::traceEverOn(true);
        trace = new VerilatedVcdC();
        top->trace(trace, 99); // TODO: What is this param?
        trace->open(argv[2]);
    }

    Env env =
    {
        .get_reset_n = get_reset_n,
        .set_reset_n = set_reset_n,
        .get_clk = get_clk,
        .set_clk = set_clk,

        .get_program_rom_addr = get_program_rom_addr,
        .set_program_rom_addr = set_program_rom_addr,
        .get_program_rom_q = get_program_rom_q,
        .set_program_rom_q = set_program_rom_q,

        .get_leds = get_leds,
        .set_leds = set_leds,

        .get_avl_ready = get_avl_ready,
        .set_avl_ready = set_avl_ready,
        .get_avl_burstbegin = get_avl_burstbegin,
        .set_avl_burstbegin = set_avl_burstbegin,
        .get_avl_addr = get_avl_addr,
        .set_avl_addr = set_avl_addr,
        .get_avl_rdata_valid = get_avl_rdata_valid,
        .set_avl_rdata_valid = set_avl_rdata_valid,
        .get_avl_rdata = get_avl_rdata,
        .set_avl_rdata = set_avl_rdata,
        .get_avl_wdata = get_avl_wdata,
        .set_avl_wdata = set_avl_wdata,
        .get_avl_be = get_avl_be,
        .set_avl_be = set_avl_be,
        .get_avl_read_req = get_avl_read_req,
        .set_avl_read_req = set_avl_read_req,
        .get_avl_write_req = get_avl_write_req,
        .set_avl_write_req = set_avl_write_req,
        .get_avl_size = get_avl_size,
        .set_avl_size = set_avl_size,

        .eval = eval,
        .final = final,
        .trace_dump = trace_dump,
    };

    auto ret = run(&env);

    if (trace)
    {
        trace->close();
        delete trace;
    }

    delete top;

    return ret;
}
