use std::str::from_utf8;
use anyhow::{Result, anyhow};
use smoltcp::iface::{Config, Interface, SocketSet, SocketStorage};
use smoltcp::socket::udp::{PacketBuffer as UdpPacketBuffer, PacketMetadata, Socket as UdpSocket};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};
use smoltcp::time::Instant;

use dpdk_core::{init_eal};
use dpdk_net::DpdkPort;

use exp_pause::SpinCounter;

fn main() -> Result<()> {
    // --- Конфигурация сети ---
    let my_mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let my_ip: [u8; 4] = [192, 168, 100, 99];
    let device_mac = EthernetAddress(my_mac);
    let device_ip = IpCidr::new(IpAddress::v4(my_ip[0], my_ip[1], my_ip[2], my_ip[3]), 24);

    // --- ШАГ 1: Инициализация среды DPDK (EAL) ---
    // Мы передаем аргументы явно. Это позволяет легко менять TAP-интерфейс или ядра.
    let app_name = "dpdk-udp-receiver";
    let eal_args = vec![
        "-l", "0-1",
        "--vdev", "net_tap0,iface=dpdk-tap0"
    ];

    init_eal(app_name, eal_args)
        .map_err(|e| anyhow!("Ошибка инициализации EAL: {}", e))?;

    // --- ШАГ 2: Создание пула памяти ---
    // Теперь это отдельный шаг. Один пул может использоваться многими портами.
    let mempool = dpdk_core::create_mempool("MBUF_POOL", 8191)
        .map_err(|e| anyhow!("Ошибка создания Mempool: {}", e))?;

    // --- ШАГ 3: Инициализация порта ---
    // Передаем ID порта (0) и ссылку на созданный пул памяти.
    let mut port = DpdkPort::new(0, &mempool)
        .map_err(|e| anyhow!("Ошибка инициализации порта: {}", e))?;

    // --- Настройка smoltcp ---
    let mut config = Config::new(device_mac.into());
    config.random_seed = 0x12345678;

    let mut iface = Interface::new(config, &mut port, Instant::from_millis(0));
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs.push(device_ip).unwrap();
    });

    iface.routes_mut().add_default_ipv4_route(Ipv4Address::new(192, 168, 100, 1)).unwrap();

    // Создаем UDP сокет
    let mut udp_rx_meta = [PacketMetadata::EMPTY; 16];
    let mut udp_rx_data = [0u8; 65535];
    let mut udp_tx_meta = [PacketMetadata::EMPTY; 16];
    let mut udp_tx_data = [0u8; 65535];
    let udp_rx_buffer = UdpPacketBuffer::new(&mut udp_rx_meta[..], &mut udp_rx_data[..]);
    let udp_tx_buffer = UdpPacketBuffer::new(&mut udp_tx_meta[..], &mut udp_tx_data[..]);
    let mut udp_socket = UdpSocket::new(udp_rx_buffer, udp_tx_buffer);

    // Слушаем на всех адресах, порт 9999
    udp_socket.bind(9999).unwrap();

    let mut socket_storage = [SocketStorage::EMPTY; 8];
    let mut sockets = SocketSet::new(&mut socket_storage[..]);
    let udp_handle = sockets.add(udp_socket);

    println!("Успех! Локальный DPDK с использованием smoltcp запущен.");
    println!("Слушаю интерфейс dpdk-tap0... Нажмите Ctrl+C для выхода.");

    let start_time = std::time::Instant::now();
    let mut pauser = SpinCounter::<1024>::new();

    loop {
        let now = std::time::Instant::now();
        let timestamp = Instant::from_millis((now - start_time).as_millis() as i64);

        iface.poll(timestamp, &mut port, &mut sockets);

        let socket = sockets.get_mut::<UdpSocket>(udp_handle);
        if let Ok((data, endpoint)) = socket.recv() {
            println!(" Пойман UDP Пакет! От: {}. Размер: {} байт", endpoint, data.len());

            if let Ok(text) = from_utf8(&data) {
                println!("   Текст: \"{}\"", text.trim());
            }

            pauser.reset();
        } else {
            pauser.pause();
        }
    }
}
