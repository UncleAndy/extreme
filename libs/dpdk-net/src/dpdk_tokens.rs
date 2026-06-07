use dpdk_sys::bindings as dpdk;
use dpdk_core::PacketBuffer;
use smoltcp::phy;
use crate::dpdk_port::DpdkPort;

pub struct DpdkRxToken {
    pub(crate) packet: PacketBuffer,
}

impl phy::RxToken for DpdkRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(self.packet.as_slice())
    }
}

pub struct DpdkTxToken<'a> {
    pub(crate) port: &'a DpdkPort,
}

impl<'a> phy::TxToken for DpdkTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        unsafe {
            let raw_mbuf = dpdk::wrap_rte_pktmbuf_alloc(self.port.mempool);
            if raw_mbuf.is_null() {
                panic!("Failed to allocate mbuf for transmission");
            }

            let mut packet = PacketBuffer { raw: raw_mbuf, mempool: self.port.mempool };
            // Устанавливаем длину данных
            (*packet.raw).data_len = len as u16;
            (*packet.raw).pkt_len = len as u32;

            let result = f(packet.as_mut_slice());
            self.port.tx_send(packet);
            result
        }
    }
}
