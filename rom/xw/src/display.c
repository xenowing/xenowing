#include <xw/inttypes.h>
#include <xw/bool.h>
#include <xw/cpu.h>
#include <xw/display.h>

#define DISPLAY_STATUS ((volatile uint32_t *)0x22000000)

#define DISPLAY_STATUS_VBLANK_BIT 0
#define DISPLAY_STATUS_VBLANK_MASK (1 << DISPLAY_STATUS_VBLANK_BIT)

#define DISPLAY_FRAMEBUFFER_ADDR ((volatile uint32_t *)0x22000004)

#define DISPLAY_I2C_READ ((volatile uint8_t *)0x22000008)
#define DISPLAY_I2C_WRITE ((volatile uint8_t *)0x2200000c)

#define DISPLAY_I2C_CLOCK_BIT 0
#define DISPLAY_I2C_DATA_BIT 1
#define DISPLAY_I2C_CLOCK_MASK (1 << DISPLAY_I2C_CLOCK_BIT)
#define DISPLAY_I2C_DATA_MASK (1 << DISPLAY_I2C_DATA_BIT)
#define DISPLAY_I2C_CLOCK_LOW 0
#define DISPLAY_I2C_CLOCK_HIGH DISPLAY_I2C_CLOCK_MASK
#define DISPLAY_I2C_DATA_LOW 0
#define DISPLAY_I2C_DATA_HIGH DISPLAY_I2C_DATA_MASK

#define I2C_DEVICE_ADDR 0x72
#define I2C_WRITE_BIT 0x00
#define I2C_READ_BIT 0x01

typedef uint8_t i2c_reg_t;

#define REG_POWER_DOWN 0x41
#define REG_POWER_DOWN_DISABLE (0 << 0)
#define REG_POWER_DOWN_ENABLE (1 << 0)

#define REG_HPD_MONITOR_SENSE 0x42
#define REG_HPD_MONITOR_SENSE_HPD_MASK (1 << 6)

static uint16_t framebuffer0[XW_FRAMEBUFFER_WIDTH * XW_FRAMEBUFFER_HEIGHT] __attribute__((aligned(8)));
static uint16_t framebuffer1[XW_FRAMEBUFFER_WIDTH * XW_FRAMEBUFFER_HEIGHT] __attribute__((aligned(8)));
static uint16_t *back_buffer;
static uint16_t *front_buffer;

void i2c_delay()
{
    // Each tick is 1/4 the I2C clock rate, which should be 400khz max
    xw_sleep_cycles(100000000 / 400000 / 4);
}

void i2c_start()
{
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_HIGH | DISPLAY_I2C_DATA_HIGH;
    i2c_delay();
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_HIGH | DISPLAY_I2C_DATA_LOW;
    i2c_delay();
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_LOW | DISPLAY_I2C_DATA_LOW;
    i2c_delay();
}

void i2c_stop()
{
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_LOW | DISPLAY_I2C_DATA_LOW;
    i2c_delay();
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_HIGH | DISPLAY_I2C_DATA_LOW;
    i2c_delay();
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_HIGH | DISPLAY_I2C_DATA_HIGH;
    i2c_delay();
}

void i2c_write_bit(uint8_t value)
{
    uint8_t data_bit = value ? DISPLAY_I2C_DATA_HIGH : DISPLAY_I2C_DATA_LOW;
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_LOW | data_bit;
    i2c_delay();
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_HIGH | data_bit;
    // Wait for potential clock stretching
    while (!(*DISPLAY_I2C_READ & DISPLAY_I2C_CLOCK_MASK))
        ;
    i2c_delay();
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_LOW | data_bit;
    i2c_delay();
}

uint8_t i2c_read_bit()
{
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_LOW | DISPLAY_I2C_DATA_HIGH;
    i2c_delay();
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_HIGH | DISPLAY_I2C_DATA_HIGH;
    // Wait for potential clock stretching
    while (!(*DISPLAY_I2C_READ & DISPLAY_I2C_CLOCK_MASK))
        ;
    i2c_delay();
    uint8_t bit = (*DISPLAY_I2C_READ & DISPLAY_I2C_DATA_MASK) ? 1 : 0;
    *DISPLAY_I2C_WRITE = DISPLAY_I2C_CLOCK_LOW | DISPLAY_I2C_DATA_HIGH;
    i2c_delay();

    return bit;
}

void i2c_write_byte(uint8_t value)
{
    for (int i = 0; i < 8; i++)
    {
        i2c_write_bit((value >> 7) & 1);
        value <<= 1;
    }

    i2c_read_bit(); // TODO: Handle ACK
}

uint8_t i2c_read_byte(bool ack)
{
    uint8_t value = 0;
    for (int i = 0; i < 8; i++)
        value = (value << 1) | i2c_read_bit();

    i2c_write_bit(ack ? 0 : 1);

    return value;
}

void i2c_set(i2c_reg_t reg, uint8_t value)
{
    i2c_start();
    i2c_write_byte(I2C_DEVICE_ADDR | I2C_WRITE_BIT);
    i2c_write_byte((uint8_t)reg);
    i2c_write_byte(value);
    i2c_stop();
}

uint8_t i2c_get(i2c_reg_t reg)
{
    i2c_start();
    i2c_write_byte(I2C_DEVICE_ADDR | I2C_WRITE_BIT);
    i2c_write_byte((uint8_t)reg);
    i2c_start();
    i2c_write_byte(I2C_DEVICE_ADDR | I2C_READ_BIT);
    uint8_t value = i2c_read_byte(false);
    i2c_stop();

    return value;
}

void adv7513_init()
{
    // Wait 200ns for adv7513 power-up
    xw_sleep_cycles(30000000);

    // Wait for HPD to be high
    while (!(i2c_get(REG_HPD_MONITOR_SENSE) & REG_HPD_MONITOR_SENSE_HPD_MASK))
        ;

    // Disable power down
    i2c_set(REG_POWER_DOWN, REG_POWER_DOWN_DISABLE);

    // Set fixed registers
    i2c_set(0x98, 0x03);
    i2c_set(0x9a, 0xe0);
    i2c_set(0x9c, 0x30);
    i2c_set(0x9d, 0x01);
    i2c_set(0xa2, 0xa4);
    i2c_set(0xa3, 0xa4);
    i2c_set(0xe0, 0xd0);
    i2c_set(0xf9, 0x00);

    // TODO: If the monitor is disconnected, how do we detect it? How much do we have to do to power on again? Can we detect it without polling the I2C port?
    // TODO: Audio? We won't have that for awhile, but probably want to generalize this to HDMI rather than just display
}

void xw_display_init()
{
    //adv7513_init();

    back_buffer = framebuffer0;
    front_buffer = framebuffer1;
    //*DISPLAY_FRAMEBUFFER_ADDR = (uint32_t)front_buffer;
}

uint16_t *xw_get_back_buffer()
{
    return back_buffer;
}

void xw_swap_buffers(bool vsync)
{
    uint16_t *temp = back_buffer;
    back_buffer = front_buffer;
    front_buffer = temp;
    /*while (vsync && !(*DISPLAY_STATUS & DISPLAY_STATUS_VBLANK_MASK))
        ;
    *DISPLAY_FRAMEBUFFER_ADDR = (uint32_t)front_buffer;*/
}
