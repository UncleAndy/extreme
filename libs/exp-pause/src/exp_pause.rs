use std::hint::spin_loop;
use std::cmp::min;

#[repr(transparent)]
pub struct SpinCounter<const LIMIT: u32> {
    // Храним количество шагов (экспоненту). u8 здесь идеален,
    // так как сдвиг 1u32 << 31 — это уже предел для u32.
    pub steps: u8,
}

impl<const LIMIT: u32> Default for SpinCounter<LIMIT> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<const LIMIT: u32> SpinCounter<LIMIT> {
    /// Создает новый счетчик с нулевого шага
    #[inline(always)]
    pub const fn new() -> Self {
        Self { steps: 0 }
    }

    /// Выполняет экспоненциальное количество инструкций PAUSE до лимита LIMIT
    #[inline(always)]
    pub fn pause(&mut self) {
        // 1. Безопасно сдвигаем 1u32 влево на количество шагов.
        // Если steps >= 32, checked_shl вернет None, и unwrap_or(u32::MAX)
        // защитит нас от паники, выдав максимальное число.
        let raw_count = 1u32.checked_shl(self.steps as u32).unwrap_or(u32::MAX);

        // 2. Ограничиваем константным лимитом LIMIT.
        // Так как LIMIT известен при компиляции, компилятор заменит его жестким числом.
        let pause_count = min(raw_count, LIMIT);

        // 3. Быстрый in-place цикл
        for _ in 0..pause_count {
            spin_loop();
        }

        // 4. Увеличиваем шаг, защищая u8 от переполнения
        self.steps = self.steps.saturating_add(1);
    }

    /// Сбрасывает экспоненту в ноль
    #[inline(always)]
    pub fn reset(&mut self) {
        self.steps = 0;
    }
}
