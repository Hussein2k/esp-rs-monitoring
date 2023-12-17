const SENSOR_TYPE_HIMIDITY = 1;
const SENSOR_TYPE_TEMPERATURE = 2;
const SENSOR_TYPE_H_T = SENSOR_TYPE_HIMIDITY + SENSOR_TYPE_TEMPERATURE;
const SENSOR_TYPE_VOLTAGE = 4;
const RELAY_MODE_AUTO = 0;
const RELAY_MODE_ALWAYS_ON = 1;
const RELAY_MODE_ALWAYS_OFF = 2;
const sensors_get_api = "http://192.168.3.79/get_sensors";
function get_data() {
    return "[{\"sensor_id\":2,\"sensor_data\":{\"sensor_type\":3,\"himidity\":52.659607,\"temperature\":25.554352,\"voltage\":0.0},\"timestamp\":4194652988}]";
}
function compare_sensor_packets(a, b) {
    return a.sensor_id - b.sensor_id;
}
function getSensors(sensors_json) {
    const connected_sensors = JSON.parse(sensors_json);
    connected_sensors.sort(compare_sensor_packets);
    //UpdateSensors(connected_sensors);
    console.log(connected_sensors);
    return connected_sensors;
}
function generateSensorWidget(sensor_packet) {
    var html_string = ` <li><div class="sensor_value_container">
                <h3 class="node_title">Node ${sensor_packet.sensor_id.toString()}</h3>
                
                <div style="display: flex; justify-content: center;">
                <img class="sensor_image" src="sensor.png">
                </div>
                `;
    if (sensor_packet.sensor_data.sensor_type & SENSOR_TYPE_HIMIDITY) {
        html_string += `<div class="sensor_value_display">
                        <a>Humidity: </a>
                        <a id = "${"sensor" + sensor_packet.sensor_id.toString() + "_humidity"}">${sensor_packet.sensor_data.himidity.toPrecision(3).toString()}%</a>
                        </div>
                        <br>`;
    }
    if (sensor_packet.sensor_data.sensor_type & SENSOR_TYPE_TEMPERATURE) {
        html_string += `<div class="sensor_value_display" >
                        <a>Temperature: </a>
                        <a id = "${"sensor" + sensor_packet.sensor_id.toString() + "_temperature"}" > ${sensor_packet.sensor_data.temperature.toPrecision(3).toString()} deg.</a>
                        </div>`;
    }
    if (sensor_packet.sensor_data.sensor_type & SENSOR_TYPE_VOLTAGE) {
        html_string += `<div class="sensor_value_display" >
                        <a>Voltage: </a>
                        <a id = "${"sensor" + sensor_packet.sensor_id.toString() + "_voltage"}" > ${sensor_packet.sensor_data.voltage.toPrecision(3).toString()} volt</a>
                        </div>`;
    }
    html_string += `<br></div> </li >`;
    const DParse = new DOMParser();
    const doc = DParse.parseFromString(html_string, 'text/html');
    return doc.body.firstChild;
    // let sensor_widget = document.createElement('li') as HTMLLIElement;
    // sensor_widget.id = 'main_list_sensor_' + sensor_data.sensor_id.toString();
    // sensor
}
var WidgetMap = [];
function add_new_to_list(new_sensor) {
    const main_list = document.getElementById('main_list');
    const tmp = generateSensorWidget(new_sensor);
    main_list.appendChild(tmp);
    WidgetMap[new_sensor.sensor_id] = tmp;
    return tmp;
}
function disable_widget(widget) {
    widget.firstElementChild.classList.add('disabled_field');
    widget.classList.remove('clickable');
    // widget.children[0].classList.add('disabled_field');
}
function updateSensorsWidget(sensors) {
    for (var i = 0; i < sensors.length; i++) {
        if (WidgetMap[sensors[i].sensor_id] == null) {
            add_new_to_list(sensors[i]).classList.add('clickable');
        }
    }
    for (const [key, value] of Object.entries(WidgetMap)) {
        var found_match = false;
        for (var i = 0; i < sensors.length; i++) {
            console.log('oF' + i.toString());
            if (parseInt(key) == sensors[i].sensor_id) {
                if (value.firstElementChild.classList.contains('disabled_field')) {
                    value.firstElementChild.classList.remove('disabled_field');
                    value.classList.add('clickable');
                }
                found_match = true;
                break;
            }
        }
        if (!found_match) {
            console.log('no match!');
            disable_widget(value);
        }
    }
}
// function configure_relay(){
//     let kk: RelayConfiguration = { sensor_id: 1, mode: 1, auto_dependence: 1, max_value: 2.2, min_value: 10.3 };
//     let json = JSON.stringify(kk);
// }
function init() {
    fetch("/get_sensors").then((res) => {
        //const sensors = getSensors("[{\"sensor_id\":2,\"sensor_data\":{\"HumidityAndTemperature\":[60.916138,23.986893]},\"time_stamp\":6572980}]");
        res.text().then((val) => {
            const sensors = getSensors(val);
            updateSensorsWidget(sensors);
            for (var i = 0; i < sensors.length; i++) {
                const humidity_a = document.getElementById('sensor' + sensors[i].sensor_id.toString() + '_humidity');
                const temprerature_a = document.getElementById('sensor' + sensors[i].sensor_id.toString() + '_temperature');
                const voltage_a = document.getElementById('sensor' + sensors[i].sensor_id.toString() + '_voltage');
                humidity_a.textContent = sensors[i].sensor_data.himidity.toPrecision(3).toString() + ' %';
                temprerature_a.textContent = sensors[i].sensor_data.temperature.toPrecision(3).toString() + ' deg.';
                voltage_a.textContent = sensors[i].sensor_data.voltage.toPrecision(3).toString() + ' volt';
            }
        });
    });
}
window.onload = init;
// setInterval(() => { console.log(10) }, 5000);
setInterval(() => {
    fetch("/get_sensors").then((res) => {
        //const sensors = getSensors("[{\"sensor_id\":2,\"sensor_data\":{\"HumidityAndTemperature\":[60.916138,23.986893]},\"time_stamp\":6572980}]");
        res.text().then((val) => {
            const sensors = getSensors(val);
            updateSensorsWidget(sensors);
            // for (var i = 0; i < sensors.length; i++) {
            //     const humidity_a = document.getElementById('sensor' + sensors[i].sensor_id.toString() + '_humidity') as HTMLAnchorElement;
            //     const temprerature_a = document.getElementById('sensor' + sensors[i].sensor_id.toString() + '_temperature') as HTMLAnchorElement;
            //     humidity_a.textContent = sensors[i].sensor_data.HumidityAndTemperature[0].toPrecision(3).toString() + '%';
            //     temprerature_a.textContent = sensors[i].sensor_data.HumidityAndTemperature[1].toPrecision(3).toString() + ' deg.';
            // }
            //console.log(sensors)
            for (var i = 0; i < sensors.length; i++) {
                const humidity_a = document.getElementById('sensor' + sensors[i].sensor_id.toString() + '_humidity');
                const temprerature_a = document.getElementById('sensor' + sensors[i].sensor_id.toString() + '_temperature');
                const voltage_a = document.getElementById('sensor' + sensors[i].sensor_id.toString() + '_voltage');
                //console.log('sensor' + sensors[i].sensor_id.toString() + '_voltage');
                if (humidity_a) {
                    humidity_a.textContent = sensors[i].sensor_data.himidity.toPrecision(3).toString() + ' %';
                }
                if (temprerature_a) {
                    temprerature_a.textContent = sensors[i].sensor_data.temperature.toPrecision(3).toString() + ' deg.';
                }
                if (voltage_a) {
                    voltage_a.textContent = sensors[i].sensor_data.voltage.toPrecision(3).toString() + ' volt';
                }
            }
        });
    });
}, 500);
//Relay Field
function enable_ranges_list(id) {
    const element = document.getElementById(id);
    if (element) {
        element.style.display = "block";
    }
}
function disable_ranges_list(id) {
    const element = document.getElementById(id);
    if (element) {
        element.style.display = "none";
    }
}
function update_relay_status(relay_number, current_mode, current_status_is_ON) {
    const mode_elem = document.getElementById('relay' + relay_number + '_mode');
    const status_elem = document.getElementById('relay' + relay_number + '_status');
    switch (current_mode) {
        case RELAY_MODE_AUTO:
            mode_elem.textContent = "Auto";
            break;
        case RELAY_MODE_ALWAYS_ON:
            mode_elem.textContent = "Always ON";
            break;
        case RELAY_MODE_ALWAYS_OFF:
            mode_elem.textContent = "Always OFF";
            break;
    }
    if (current_status_is_ON) {
        status_elem.textContent = "ON";
    }
    else {
        status_elem.textContent = "OFF";
    }
}
function get_relay_configuration(relay_number) {
    let relay_conf = { sensor_id: 0, relay_id: 0, mode: 0, min_value: 0, max_value: 0, auto_dependence: 0 };
    var relay_mode_element = document.querySelectorAll('input[type="radio"][name="r' + relay_number.toString() + 'g1"]:checked');
    const relay_mode = relay_mode_element[0].attributes['value'];
    console.log(relay_mode_element);
    const sensor_sel_element = document.getElementById('relay_conf_sensor_sel_' + relay_number.toString());
    const sensor_dep_element = document.getElementById('relay_conf_dep_sel_' + relay_number.toString());
    const min = document.getElementById('min' + relay_number.toString());
    const max = document.getElementById('max' + relay_number.toString());
    relay_conf.sensor_id = parseInt(sensor_sel_element.value);
    relay_conf.relay_id = relay_number;
    relay_conf.mode = parseInt(relay_mode.textContent);
    relay_conf.auto_dependence = parseInt(sensor_dep_element.value);
    relay_conf.min_value = parseFloat(min.value);
    relay_conf.max_value = parseFloat(max.value);
    console.log(relay_conf);
    // Send JSON data to the backend
    fetch('http://192.168.71.1/set_relays', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(relay_conf)
    })
        .then(response => response.json())
        .then(data => {
        console.log('Response from backend:', data);
        // Handle the response from the backend
    })
        .catch(error => {
        console.error('Error:', error);
        // Handle errors
    });
    return relay_conf;
}
function SetRange(n) {
    // const elemes = document.querySelectorAll('input[name = "group1"][checked="true"]')
    // console.log('vioala')
    // if (elemes) {
    //     console.log(elemes)
    // }
    // let relay_conf: RelayConfiguration = { sensor_id: 1, mode: 1, auto_dependence: 1, max_value: 2.2, min_value: 10.3 };
    // let json = JSON.stringify(kk);
    var relay_mode_element = document.querySelectorAll('input[type="radio"][name="r' + n.toString() + 'g1"]:checked');
    const relay_mode = relay_mode_element[0].attributes['value'];
    console.log(relay_mode);
    if (relay_mode.textContent == "val_auto") {
        console.log('found_t');
        var auto_mode_type = document.querySelectorAll('input[type="radio"][name="r' + n.toString() + 'g2"]:checked');
        const v = auto_mode_type[0].attributes['value'];
        const min = document.getElementById('min' + n.toString());
        const max = document.getElementById('max' + n.toString());
        console.log('min' + n.toString());
        console.log('auto_mode_type :');
        console.log(min.value);
    }
    // console.log(relay_mode_element[0].attributes['value'])
}
//POST
// function post_t() {
//     var jsonData = {
//         key1: "value1",
//         key2: "value2"
//     };
//     // Send JSON data to the backend
//     fetch('http://192.168.71.1/set_relays', {
//         method: 'POST',
//         headers: {
//             'Content-Type': 'application/json'
//         },
//         body: JSON.stringify(jsonData)
//     })
//         .then(response => response.json())
//         .then(data => {
//             console.log('Response from backend:', data);
//             // Handle the response from the backend
//         })
//         .catch(error => {
//             console.error('Error:', error);
//             // Handle errors
//         });
// }
