use std::collections::VecDeque;
use std::ptr;
use smoltcp::phy::{Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use dpdk_sys::bindings as dpdk;
use dpdk_core::PacketBuffer;
use dpdk_core::Mempool;
use crate::dpdk_tokens::{DpdkRxToken, DpdkTxToken};

// Безопасная обертка над сетевым портом
pub struct DpdkPort {
    id: u16,
    rx_queue: VecDeque<PacketBuffer>,
    pub(crate) mempool: *mut dpdk::rte_mempool,
}

impl DpdkPort {
    /// Инициализирует сетевой порт DPDK.
    ///
    /// # Аргументы
    /// * `port_id` - ID порта (обычно 0 для первого устройства).
    /// * `mempool` - Ссылка на уже созданный пул памяти (из dpdk-core).
    pub fn new(port_id: u16, mempool: &Mempool) -> Result<Self, String> {
        unsafe {
            // 1. Конфигурация порта
            let port_conf: dpdk::rte_eth_conf = std::mem::zeroed();
            if dpdk::rte_eth_dev_configure(port_id, 1, 1, &port_conf) < 0 {
                return Err(format!("Ошибка конфигурации порта {}: rte_eth_dev_configure", port_id));
            }

            // --- ИСПРАВЛЕНИЕ ОШИБОК ТУТ ---

            // Исправление ошибки: "This function takes 6 parameters, but 5 parameters were supplied"
            // Сигнатура rte_eth_rx_queue_setup в новых DPDK:
            // (port_id, queue_id, nb_desc, rx_conf, mempool) -> в некоторых версиях может быть иначе.
            // Если bindgen говорит, что нужно 6 параметров, значит ваша версия DPDK требует дополнительный аргумент.
            // Обычно это: port_id, queue_id, nb_desc, rx_conf, mempool, и иногда дополнительные флаги.

            // Создаем пустую структуру конфигурации RX, чтобы передать её указатель
            let rx_conf: dpdk::rte_eth_rxconf = std::mem::zeroed();

            // Исправление ошибок Type Mismatch:
            // 1. port_id должен быть приведен к c_uint (обычно u32)
            // 2. rx_conf передается как указатель &rx_conf
            // 3. mempool.raw передается как указатель на пул

            if dpdk::rte_eth_rx_queue_setup(
                port_id,               // 1. port_id: u16
                0,                     // 2. rx_queue_id: u16 (обычно 0 для первой очереди)
                1024,                  // 3. nb_rx_desc: u16 (размер кольца дескрипторов)
                0,                     // 4. socket_id: c_uint (0 означает "использовать текущий сокет")
                &rx_conf as *const _,  // 5. rx_conf: *const rte_eth_rxconf (указатель на конфиг)
                mempool.raw,           // 6. mb_pool: *mut rte_mempool (указатель на пул памяти)
            ) < 0 {
                return Err(format!("Ошибка настройки RX очереди порта {}", port_id));
            }

            let tx_conf: dpdk::rte_eth_txconf = std::mem::zeroed();

            if dpdk::rte_eth_tx_queue_setup(
                port_id,               // 1. port_id: u16
                0,                     // 2. tx_queue_id: u16
                1024,                  // 3. nb_tx_desc: u16
                0,                     // 4. socket_id: c_uint (автоматический выбор сокета)
                &tx_conf as *const _,  // 5. tx_conf: указатель на конфигурацию TX
            ) < 0 {
                return Err(format!("Ошибка настройки TX очереди порта {}", port_id));
            }

            // -----------------------------

            if dpdk::rte_eth_dev_start(port_id) < 0 {
                return Err(format!("Ошибка запуска порта {}: rte_eth_dev_start", port_id));
            }

            Ok(DpdkPort {
                id: port_id,
                rx_queue: VecDeque::new(),
                mempool: mempool.raw
            })
        }
    }

    pub fn rx_burst(&self, max_packets: usize) -> Vec<PacketBuffer> {
        let mut raw_mbufs: Vec<*mut dpdk::rte_mbuf> = vec![ptr::null_mut(); max_packets];
        unsafe {
            // Вызов Си-обертки из dpdk-sys
            let nb_rx = dpdk::wrap_rte_eth_rx_burst(self.id, 0, raw_mbufs.as_mut_ptr(), max_packets as u16);

            raw_mbufs.into_iter()
                .take(nb_rx as usize)
                .filter(|ptr| !ptr.is_null())
                // PacketBuffer теперь импортируется из dpdk-core
                .map(|ptr| PacketBuffer {
                    raw: ptr,
                    mempool: self.mempool
                })
                .collect()
        }
    }

    pub fn tx_send(&self, packet: PacketBuffer) -> bool {
        unsafe {
            let mut tx_buffer = [packet.raw; 1];
            let nb_tx = dpdk::wrap_rte_eth_tx_burst(self.id, 0, tx_buffer.as_mut_ptr(), 1);
            if nb_tx > 0 {
                // Забываем про владение, так как памятью теперь управляет Си-драйвер
                std::mem::forget(packet);
                true
            } else {
                false
            }
        }
    }
}

impl Device for DpdkPort {
    type RxToken<'a> = DpdkRxToken;
    type TxToken<'a> = DpdkTxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.rx_queue.is_empty() {
            let packets = self.rx_burst(32);
            self.rx_queue.extend(packets);
        }

        self.rx_queue.pop_front().map(|packet| {
            let rx = DpdkRxToken { packet };
            let tx = DpdkTxToken { port: self };
            (rx, tx)
        })
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(DpdkTxToken { port: self })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1500;
        caps.medium = Medium::Ethernet;
        caps.max_burst_size = Some(32);
        caps
    }
}
