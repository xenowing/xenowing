typedef char int8_t;
typedef unsigned char uint8_t;
typedef short int16_t;
typedef unsigned short uint16_t;
typedef long int32_t;
typedef unsigned long uint32_t;
typedef long long int64_t;
typedef unsigned long long uint64_t;

static volatile uint8_t *XENOWING_LEDS = (uint8_t *)0x20000000;

extern uint64_t xenowing_cycles();

int main()
{
    uint8_t leds = 5;

    while (1)
    {
        *XENOWING_LEDS = leds;
        leds -= 1;

        uint64_t t = xenowing_cycles();
        while (xenowing_cycles() - t < 20000000)
            ;
    }

    return 0;
}
