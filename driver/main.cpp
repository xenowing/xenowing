#include "../obj_dir/Valu.h"

#include <iostream>
using namespace std;

int main()
{
    Valu top;

    // Reset
    top.rst = top.clk = 0;
    top.eval();
    top.rst = 1;

    for (int time = 0; time < 10 && !Verilated::gotFinish(); time++)
    {
        cout << "out: " << top.out << endl;
        
        // Rising edge
        top.clk = 1;
        top.eval();

        // Falling edge
        top.clk = 0;
        top.eval();
    }
    top.final();

    return 0;
}
