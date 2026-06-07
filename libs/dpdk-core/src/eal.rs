use dpdk_sys::bindings as dpdk;
use std::ffi::CString;


/// Инициализирует среду выполнения DPDK (Environment Abstraction Layer).
///
/// # Аргументы
/// * `app_name` - Имя вашего приложения (появится в логах DPDK).
/// * `eal_args` - Список аргументов для EAL.
///                Например: vec!["-l", "0-1", "--vdev", "net_tap0,iface=tap0"]
pub fn init_eal(app_name: &str, eal_args: Vec<&str>) -> Result<(), String> {
    // 1. Формируем полный список аргументов.
    // Первый аргумент в DPDK всегда должен быть именем приложения.
    let mut full_args = vec![app_name];
    full_args.extend(eal_args);

    // 2. Конвертируем Rust-строки (&str) в C-строки (CString)
    // Это необходимо, так как DPDK ожидает массив указателей на null-terminated строки.
    let c_args: Vec<CString> = full_args
        .iter()
        .map(|&s| CString::new(s).unwrap_or_else(|_| CString::new("invalid").unwrap()))
        .collect();

    // 3. Получаем указатели на данные этих строк
    let mut arg_ptrs: Vec<*mut std::os::raw::c_char> = c_args
        .iter()
        .map(|c| c.as_ptr() as *mut _ )
        .collect();

    unsafe {
        // 4. Вызываем саму функцию инициализации DPDK.
        // rte_eal_init возвращает 0 при успехе и отрицательное число при ошибке.
        if dpdk::rte_eal_init(arg_ptrs.len() as i32, arg_ptrs.as_mut_ptr()) < 0 {
            return Err("Ошибка при инициализации DPDK EAL. Проверьте параметры запуска и права доступа".to_string());
        }
    }

    Ok(())
}
