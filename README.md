# Reading a TMP36 temperature sensor with a blue pill and Rust

This is a first attempt at reading an analog temperature sensor (the TMP36)
with a blue pill, using the programming language Rust. I also used a UART to
USB adapter to get the readings through serial. Since the documentation is
quite sparse for this combination (blue pill + Rust + analog sensor), I
decided to publish what I got so far.

For the environment setting, see
[here](https://github.com/JPFrancoia/bluepill_quick_start).


## Key points

### Wiring

![](wiring.jpg "")

### Reading the temperature

```rust
    // Read the TMP36's tension, in bytes
    let data: u16 = adc1.read(&mut tmp36).unwrap();
```

The TMP36's tension is read from an analog pin. The ADC (Analog to Digital
Converter) returns a tension, in bytes. This value needs to be converted to a
tension (in mV), and then to a temperature, depending on the sensor's specs.

The blue pill has a 12 bits ADC, so the values it can return range from 0 to
4095. 4095 corresponds to ~3.3 V. I have no idea if it's possible to specify an
internal/external reference (I didn't need it in this case) for the conversion.

Once I got a tension in mV, I needed to convert it to a temperature. The TMP36
can read temperatures from -50°C to 125°C. The tension it outputs is directly
proportional to the temperature. At -50°C, the sensor returns ~0 mV. At 125°C,
it outputs 1750 mV.

All the conversion calculations was put into a function:

```rust
fn scaling(raw_value: u16) -> f32 {

    // 12 bits -> 4095
    // ref = 3.3V
    // value read from analog pin -> 4095
    // ? -> 3.3V
    let v_ref = 3300;
    let raw_tension = raw_value as f32 * v_ref as f32 / 4095.0;

    // Sensor can measure temperature from -50°C to 125°C
    // 125 + 50 = 175 -> we need that to cover the full range of temperatures
    let temperature = raw_tension as f32 * 175.0 / 1750.0 - 50.0;

    return temperature;
}
```

### Sending the temperature over serial

Once I managed to read the temperature, I decided to send the reading through
serial. The first obstacle was concerting a float to a string. As weird as it
seems, it's not a trivial operation in `no_std` rust, I had to use the `ryu`
external crate.

And then, here is the code to send the reading though serial:

```rust
    for byte in printed.bytes() {
        while block!(tx.write(byte)).is_err(){};
    }

    for byte in [b"\r", b"\n"].iter() {
        while block!(tx.write(byte[0])).is_err(){};
    }

    tx.flush();
```

We basically iterate over each byte of the stringified temperature and send
it through serial. **NOTE**: this is probably not the simplest way to do it, I
heard there are crates to do that easily.
