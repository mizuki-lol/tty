# Převodník RS-232 na ITA-2

Tento projekt slouží k připojení zařízení pomocí RS-232 přes sériový kabel k dálnopisné síti.


## Konstrukce

K sestavení převodníku je potřeba:
  * Raspberry Pi Pico 1
  * Převodník RS-232 - TTL
  * 2x optočlen
  * 3x 300 Ohm rezistor
  * Nepájivé pole
  * Dráty pro propojení součástek na nepájivém poli

## Kompilace & flashování

Závislosti:
  * [rustpup](https://rustup.rs/)    
  * Rust - verze 1.75 a vyšší
    * Nainstaluje spolu s rustupem
  * thumbv6m-none-eabi target pro rustc
    * lze získat tímto příkazem: `rustup target add thumbv6m-none-eabi`
  * elf2uf2-rs
    * lze získat tímto příkazem: `cargo install elf2uf2-rs`

```
# Kompilace projektu
cargo build --release

# Převedení na uf2
elf2uf2-rs target/thumbv6m-none-eabi/release/tty tty.uf2

# Nahrání na Rpi Pico.
# Připojte Rpi Pico k počítači pomocí Micro-usb kabelu zatímco držíte tlačítko BOOTSEL.
# (Tento krok není nutné dělat přes příkazový řádek a je možné ho udělat
# pomocí správce souborů.)
cp tty.uf2 <cesta k rpi> # Zaměňte <cesta k rpi> za složku kde se nachází Rpi Pico.
```

## Použití

Připojte k Rpi Picu zdroj napětí pomocí Micro-usb kabelu.
Připojte drát napěťové smyčky dálnopisu na příslušné piny.
Připojte svůj počítač sériovým kabelem k převodníku.
Zřiďte připojění k sériovému portu pomocí programu jako např. puTTY s nastavením:
  * Speed (baud) - 300
  * Data bits - 8
  * Stop bits - 1
  * Parity - None
  * Flow control - None  

Nyní můžete psát text do puTTY, který se pošle dálnopisu.
Stejným způsobem se text poslaný dálnopisem zobrazí v puTTY.
