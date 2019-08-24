#include <xw/inttypes.h>
#include <xw/display.h>

#define I2C_ADDR 0x72

typedef uint8_t i2c_reg_t;

#define REG_POWER_DOWN 0x41
#define REG_POWER_DOWN_DISABLE 0x00
#define REG_POWER_DOWN_ENABLE 0x01

void i2c_set(i2c_reg_t reg, uint8_t value)
{
    // TODO
}

uint8_t i2c_get(i2c_reg_t reg)
{
    // TODO
    return 0;
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

void xw_display_init()
{
    adv7513_init();

    // TODO: Enable video output data generation (should that be a separate unit?)
}
