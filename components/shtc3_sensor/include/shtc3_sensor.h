
#ifndef SHTC3_SENSOR_HH
#define SHTC3_SENSOR_HH
#include <driver/gpio.h>
#include <driver/i2c.h>
#include <esp_err.h>
#include <esp_log.h>

#define SENSOR_ADDR 0x70
#define T_FIRST_N_CS 0x7CA2
#define RH_FIRST_N_CS 0x5C24
#define T_FIRST_LP_CS 0x6458
#define RH_FIRST_LP_CS 0x44DE

#define T_FIRST_N 0x7866
#define RH_FIRST_N 0x58E0
#define T_FIRST_LP 0x609C
#define RH_FIRST_LP 0x401A

#define I2C_MASTER_FREQ_100KHZ 100000

#if __cplusplus
extern "C"
{
#endif
    void shtc3_init(int sda_pin, int scl_pin, int frequency);
    esp_err_t shtc3_wake_up(uint8_t addr);
    esp_err_t shtc3_read(uint8_t addr, uint16_t read_mode, float *temp, float *hum);
    esp_err_t shtc3_sleep(uint8_t addr);
#if __cplusplus
}
#endif
#endif