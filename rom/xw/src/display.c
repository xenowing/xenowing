#include <xw/inttypes.h>
#include <xw/cpu.h>
#include <xw/display.h>

#define XW_DISPLAY_I2C_READ ((volatile uint8_t *)0x22000000)
#define XW_DISPLAY_I2C_WRITE ((volatile uint8_t *)0x22000004)

// TODO: I'm not entirely sure how I want to specify bit indices/masks for registers generally; need to see more usage examples first
#define XW_DISPLAY_I2C_CLOCK_BIT 0
#define XW_DISPLAY_I2C_DATA_BIT 1
#define XW_DISPLAY_I2C_CLOCK_MASK (1 << XW_DISPLAY_I2C_CLOCK_BIT)
#define XW_DISPLAY_I2C_DATA_MASK (1 << XW_DISPLAY_I2C_DATA_BIT)
#define XW_DISPLAY_I2C_CLOCK_LOW 0
#define XW_DISPLAY_I2C_CLOCK_HIGH XW_DISPLAY_I2C_CLOCK_MASK
#define XW_DISPLAY_I2C_DATA_LOW 0
#define XW_DISPLAY_I2C_DATA_HIGH XW_DISPLAY_I2C_DATA_MASK

#define I2C_DEVICE_ADDR 0x72
#define I2C_WRITE_BIT 0x00
#define I2C_READ_BIT 0x01

typedef uint8_t i2c_reg_t;

#define REG_POWER_DOWN 0x41
#define REG_POWER_DOWN_DISABLE 0x00
#define REG_POWER_DOWN_ENABLE 0x01

void i2c_delay(uint32_t ticks)
{
    // Each tick is 1/4 the I2C clock rate, which should be 100khz max (TODO: We can likely make this faster for the adv7513, check data sheet)
    const uint64_t cycles_per_tick = 150000000 / 100000 / 4;
    while (ticks)
    {
        xw_sleep_cycles(cycles_per_tick);
        ticks--;
    }
}

void i2c_start_condition()
{
    *XW_DISPLAY_I2C_WRITE = (*XW_DISPLAY_I2C_READ & XW_DISPLAY_I2C_CLOCK_MASK) | XW_DISPLAY_I2C_DATA_HIGH;
    i2c_delay(1);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_HIGH | XW_DISPLAY_I2C_DATA_HIGH;
    i2c_delay(1);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_HIGH | XW_DISPLAY_I2C_DATA_LOW;
    i2c_delay(1);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_LOW | XW_DISPLAY_I2C_DATA_LOW;
    i2c_delay(1);
}

void i2c_stop_condition()
{
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_LOW | XW_DISPLAY_I2C_DATA_LOW;
    i2c_delay(1);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_HIGH | XW_DISPLAY_I2C_DATA_LOW;
    i2c_delay(1);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_HIGH | XW_DISPLAY_I2C_DATA_HIGH;
    i2c_delay(1);
}

void i2c_write_bit(uint8_t value)
{
    uint8_t data_bit = value ? XW_DISPLAY_I2C_DATA_HIGH : XW_DISPLAY_I2C_DATA_LOW;
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_LOW | data_bit;
    i2c_delay(1);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_HIGH | data_bit;
    i2c_delay(2);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_LOW | data_bit;
    i2c_delay(1);
}

uint8_t i2c_read_bit()
{
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_LOW | XW_DISPLAY_I2C_DATA_HIGH;
    i2c_delay(1);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_HIGH | XW_DISPLAY_I2C_DATA_HIGH;
    i2c_delay(1);
    uint8_t bit = (*XW_DISPLAY_I2C_READ & XW_DISPLAY_I2C_DATA_MASK) ? 1 : 0;
    i2c_delay(1);
    *XW_DISPLAY_I2C_WRITE = XW_DISPLAY_I2C_CLOCK_LOW | XW_DISPLAY_I2C_DATA_HIGH;
    i2c_delay(1);

    return bit;
}

void i2c_write_byte(uint8_t value)
{
    for (int i = 0; i < 8; i++)
        i2c_write_bit((value >> (7 - i)) & 1);

    i2c_read_bit(); // TODO: Handle ACK
}

uint8_t i2c_read_byte()
{
    uint8_t value = 0;
    for (int i = 0; i < 8; i++)
        value = (value << 1) | i2c_read_bit();

    i2c_read_bit(); // TODO: Handle ACK

    return value;
}

void i2c_set(i2c_reg_t reg, uint8_t value)
{
    i2c_start_condition();
    i2c_write_byte(I2C_DEVICE_ADDR | I2C_WRITE_BIT);
    i2c_start_condition();
    i2c_write_byte((uint8_t)reg);
    i2c_start_condition();
    i2c_write_byte(value);
    i2c_stop_condition();
}

uint8_t i2c_get(i2c_reg_t reg)
{
    i2c_start_condition();
    i2c_write_byte(I2C_DEVICE_ADDR | I2C_READ_BIT);
    i2c_start_condition();
    i2c_write_byte((uint8_t)reg);
    i2c_start_condition();
    uint8_t value = i2c_read_byte();
    i2c_stop_condition();

    return value;
}

void adv7513_init()
{
    // TODO: Datasheet says to wait for a period before interacting with i2c bus, do we do that here? how long?
    // TODO: Do we need to detect if the monitor is connected before powering on the transmitter?
    // TODO: If the monitor is disconnected, how do we detect it? How much do we have to do to power on again?

    // Disable power down
    i2c_set(REG_POWER_DOWN, REG_POWER_DOWN_DISABLE);

    // Set fixed registers
    i2c_set(0x98, 0x03);
    i2c_set(0x9a, 0xe0); // TODO: This one may be wrong :P
    i2c_set(0x9c, 0x30);
    i2c_set(0x9d, 0x01);
    i2c_set(0xa2, 0xa4);
    i2c_set(0xa3, 0xa4);
    i2c_set(0xe0, 0xd0);
    i2c_set(0xf9, 0x00);

    // TODO: Set video input mode
    // TODO: Set video output mode

    // TODO: Audio? We won't have that for awhile, but probably want to generalize this to HDMI rather than just display
}

char debug_get_digit(uint8_t value)
{
    value &= 0x0f;
    if (value < 10)
        return '0' + value;
    return 'a' + (value - 10);
}

#include <xw/uart.h>

void i2c_debug_read(i2c_reg_t reg)
{
    uint8_t value = i2c_get(reg);
    char str[] = "0x00: 0x00";
    str[2] = debug_get_digit(reg >> 4);
    str[3] = debug_get_digit(reg);
    str[8] = debug_get_digit(value >> 4);
    str[9] = debug_get_digit(value);
    xw_puts(str);
}

void xw_display_init()
{
    // I2C testing for now...
    i2c_debug_read(REG_POWER_DOWN);
    i2c_set(REG_POWER_DOWN, REG_POWER_DOWN_DISABLE);
    i2c_debug_read(REG_POWER_DOWN);
    i2c_set(REG_POWER_DOWN, REG_POWER_DOWN_ENABLE);
    i2c_debug_read(REG_POWER_DOWN);
    i2c_set(REG_POWER_DOWN, REG_POWER_DOWN_DISABLE);
    i2c_debug_read(REG_POWER_DOWN);

    //adv7513_init();

    // TODO: Enable video output data generation (should that be a separate unit?)
}
