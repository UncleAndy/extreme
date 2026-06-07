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
