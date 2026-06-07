use dpdk_sys::bindings as dpdk;

// Безопасная обертка над пакетом буфера
pub struct PacketBuffer {
    pub raw: *mut dpdk::rte_mbuf,
    pub mempool: *mut dpdk::rte_mempool,
}

impl PacketBuffer {
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let data_ptr = ((*self.raw).buf_addr as *const u8).add((*self.raw).data_off as usize);
            let data_len = (*self.raw).data_len as usize;
            std::slice::from_raw_parts(data_ptr, data_len)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            let data_ptr = ((*self.raw).buf_addr as *mut u8).add((*self.raw).data_off as usize);
            let data_len = (*self.raw).data_len as usize;
            std::slice::from_raw_parts_mut(data_ptr, data_len)
        }
    }
}

impl Drop for PacketBuffer {
    fn drop(&mut self) {
        unsafe {
            dpdk::wrap_rte_pktmbuf_free_seg(self.raw);
        }
    }
}
