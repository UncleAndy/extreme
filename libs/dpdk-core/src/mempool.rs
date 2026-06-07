use std::ffi::CString;
use dpdk_sys::bindings as dpdk;

// Безопасная обертка над пулом памяти DPDK
pub struct Mempool {
    pub raw: *mut dpdk::rte_mempool,
}

/// Создает пул пакетов (MBUF pool) в DPDK.
///
/// # Аргументы
/// * `name` - Имя пула (используется DPDK для идентификации).
/// * `nb_mbufs` - Общее количество пакетов в пуле (должно быть кратно кэшу).
pub fn create_mempool(name: &str, nb_mbufs: u32) -> Result<Mempool, String> {
    // 1. Конвертируем имя пула в C-строку
    let c_name = CString::new(name)
        .map_err(|_| "Некорректное имя пула (содержит null-байты)".to_string())?;

    unsafe {
        // 2. Вызываем функцию создания пула из биндингов.
        // Аргументы:
        // - name: имя пула
        // - nb_mbufs: сколько пакетов выделить
        // - cache_size: размер локального кэша (256 — стандартное значение)
        // - socket_id: NUMA сокет (0 — автоматический выбор)
        // - data_room_size: размер дополнительного пространства в пакете (RTE_MBUF_DEFAULT_BUF_SIZE)
        // - private_data_size: размер приватных данных (0)
        let raw_pool = dpdk::rte_pktmbuf_pool_create(
            c_name.as_ptr(),
            nb_mbufs,
            256,
            0,
            dpdk::RTE_MBUF_DEFAULT_BUF_SIZE as u16,
            0
        );

        // 3. Проверяем, что пул был успешно создан
        if raw_pool.is_null() {
            return Err("Ошибка DPDK: Не удалось создать пул памяти (rte_pktmbuf_pool_create)".to_string());
        }

        // Возвращаем безопасную обертку над указателем
        Ok(Mempool { raw: raw_pool })
    }
}
