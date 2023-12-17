#include <stdio.h>
#include "shtc3_sensor.h"

void shtc3_init(int sda_pin, int scl_pin, int frequency)
{

    i2c_config_t conf = {};
    conf.mode = I2C_MODE_MASTER;
    conf.sda_io_num = (gpio_num_t)(sda_pin);
    conf.scl_io_num = (gpio_num_t)(scl_pin);

    conf.sda_pullup_en = GPIO_PULLUP_ENABLE;
    conf.scl_pullup_en = GPIO_PULLUP_ENABLE;

    conf.master.clk_speed = frequency;

    i2c_param_config(I2C_NUM_0, &conf);
    i2c_driver_install(I2C_NUM_0, conf.mode,
                       0,
                       0,
                       0);
}

esp_err_t shtc3_wake_up(uint8_t addr)
{
    const char *TAG = "WakeUp";
    i2c_cmd_handle_t cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, addr << 1 | I2C_MASTER_WRITE, true);
    i2c_master_write_byte(cmd, 0x35, true);
    i2c_master_write_byte(cmd, 0x17, true);
    i2c_master_stop(cmd);
    esp_err_t err = i2c_master_cmd_begin(I2C_NUM_0, cmd, 2);
    i2c_cmd_link_delete(cmd);
    if (err)
        ESP_LOGI(TAG, "%s", esp_err_to_name(err));
    return err;
}

esp_err_t shtc3_read(uint8_t addr, uint16_t read_mode, float *temp, float *hum)
{
    vTaskDelay(pdMS_TO_TICKS(10));
    // Read T First
    i2c_cmd_handle_t cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, addr << 1 | I2C_MASTER_WRITE, true);
    i2c_master_write_byte(cmd, 0x78, true);
    i2c_master_write_byte(cmd, 0x66, true);
    i2c_master_stop(cmd);
    esp_err_t err = i2c_master_cmd_begin(I2C_NUM_0, cmd, 2);
    if (err)
        return err;

    i2c_cmd_link_delete(cmd);

    vTaskDelay(pdMS_TO_TICKS(30));

    // Read
    uint8_t data[6];

    cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, addr << 1 | I2C_MASTER_READ, true);
    i2c_master_read(cmd, data, 6, I2C_MASTER_LAST_NACK);
    i2c_master_stop(cmd);
    err = i2c_master_cmd_begin(I2C_NUM_0, cmd, 2);
    i2c_cmd_link_delete(cmd);

    *temp = ((((data[0] << 8) + data[1]) * 175) / 65536.0) - 45;
    *hum = ((((data[3] << 8) + data[4]) * 100) / 65536.0);

    return err;
}

esp_err_t shtc3_sleep(uint8_t addr)
{
    const char *TAG = "Sleep";
    i2c_cmd_handle_t cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, addr << 1 | I2C_MASTER_WRITE, true);
    i2c_master_write_byte(cmd, 0xB0, true);
    i2c_master_write_byte(cmd, 0x98, true);
    i2c_master_stop(cmd);
    esp_err_t err = i2c_master_cmd_begin(I2C_NUM_0, cmd, 2);
    i2c_cmd_link_delete(cmd);
    if (err)
        ESP_LOGI(TAG, "%s", esp_err_to_name(err));
    return err;
}

