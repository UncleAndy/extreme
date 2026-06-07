# EXTREME library

# Overview
This library includes extreme speed and accuracy for low-latency and super-fast applications:
- dpdk-* - DPDK binding library for Rust (require dpdk-dev package);
- exp-pause - for using CPU PAUSE with exponential counter;
- udp-receiver - UDP receiver (example for DPDK);

# Запуск демо приложения `udp-receiver`

1. Запустите приложение `udp-receiver` с помощью команды (sudo обязательно):
   ```bash
   sudo cargo run -p udp-receiver
   ```

2. Поднимите внутренний IP для возможности отправки пакетов:
   ```bash
   sudo ip addr add 192.168.100.1/24 dev dpdk-tap0
   sudo ip link set dpdk-tap0 up
   ```

3. Запустите передачу пакета UDP через net-cat:
   ```bash
   echo "DPDK передает привет языку Rust!" | nc -u -w1 192.168.100.99 9999
   ```

4. Проверьте вывод приложения `udp-receiver` в консоли, чтобы убедиться, что пакет был успешно принят.

# ExpPause::SpinCounter check

При проверке цикла работы в холостом режиме на одном ядре процессора через `turbostat` после примерно 5 минут работы:
- без экспоненциального счетчика:
```
Core	CPU	Avg_MHz	Busy%	Bzy_MHz	TSC_MHz	IPC	IRQ	SMI	POLL	C1ACPI	C2ACPI	C3ACPI	POLL%	C1ACPI%	C2ACPI%	C3ACPI%	CPU%c1	CPU%c6	CPU%c7	CoreTmp	CoreThr	PkgTmp	Totl%C0Any%C0	GFX%C0	CPUGFX%	CPU%LPI	SYS%LPI	PkgWatt	CorWatt	GFXWatt	RAMWatt	PKG_%	RAM_%	UncMHz
0	0	3400	100.02	3399	3417	3.15	5013	0	0	0	0	0	0.00	0.00	0.00	0.00	0.00	0.00	0.00	58	0	58	261.67100.52	0.00	0.00	0.00	0.00	23.24	21.92	0.00	0.00	0.00	0.00	2800
```
- с экспоненциальным счетчиком в режиме `SpinCounter::<1024>::new()`:
```
Core	CPU	Avg_MHz	Busy%	Bzy_MHz	TSC_MHz	IPC	IRQ	SMI	POLL	C1ACPI	C2ACPI	C3ACPI	POLL%	C1ACPI%	C2ACPI%	C3ACPI%	CPU%c1	CPU%c6	CPU%c7	CoreTmp	CoreThr	PkgTmp	Totl%C0Any%C0	GFX%C0	CPUGFX%	CPU%LPI	SYS%LPI	PkgWatt	CorWatt	GFXWatt	RAMWatt	PKG_%	RAM_%	UncMHz
0	0	3400	100.02	3400	3417	0.38	1595	0	0	0	0	0	0.00	0.00	0.00	0.00	0.00	0.00	0.00	51	0	55	307.01100.52	0.00	0.00	0.00	0.00	22.46	21.14	0.00	0.00	0.00	0.00	2800
```

Как можно видеть, при использовании SpinCounter энергопотребление ядра снижается с 3.15W до 0.38W. При этом температура ядра снижается с 58℃ до 51℃.
