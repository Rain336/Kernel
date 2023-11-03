use super::hpet::HIGH_PRECISION_EVENT_TIMER;
use crate::counter::Counter;
use crate::CPUID;
use common::sync::SyncLazy;
use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicU64, Ordering};

pub static TIME_STAMP_COUNTER: SyncLazy<TscCounter> = SyncLazy::new(TscCounter::new);
static TSC_FREQUENCY: AtomicU64 = AtomicU64::new(0);

pub enum TscCounter {
    Invariant(u64),
    Reported,
    Calibrated,
}

impl TscCounter {
    fn new() -> Self {
        let hypervisor = CPUID
            .get_hypervisor_info()
            .and_then(|x| x.tsc_frequency())
            .map(|x| x as u64 * 1000);
        if let Some(x) = hypervisor {
            return TscCounter::Invariant(x);
        }

        let invariant_tsc = CPUID
            .get_advanced_power_mgmt_info()
            .map(|x| x.has_invariant_tsc())
            .unwrap_or_default();

        let frequency = get_reported_frequency();

        match (invariant_tsc, frequency) {
            (true, None) => TscCounter::Invariant(calibrate()),
            (true, Some(x)) => TscCounter::Invariant(x),
            (false, None) => {
                TSC_FREQUENCY.store(calibrate(), Ordering::Release);
                TscCounter::Calibrated
            }
            (false, Some(x)) => {
                TSC_FREQUENCY.store(x, Ordering::Release);
                TscCounter::Reported
            }
        }
    }
}

impl Counter for TscCounter {
    fn frequency(&self) -> u64 {
        match self {
            TscCounter::Invariant(x) => *x,
            TscCounter::Reported | TscCounter::Calibrated => TSC_FREQUENCY.load(Ordering::Acquire),
        }
    }

    fn calibrate(&self) {
        match self {
            TscCounter::Invariant(_) => {}
            TscCounter::Reported => {
                let frequency = get_reported_frequency().unwrap_or_default();
                TSC_FREQUENCY.store(frequency, Ordering::Release);
            }
            TscCounter::Calibrated => {
                let frequency = calibrate();
                TSC_FREQUENCY.store(frequency, Ordering::Release);
            }
        }
    }
}

fn get_reported_frequency() -> Option<u64> {
    CPUID.get_tsc_info().and_then(|info| {
        if info.nominal_frequency() != 0 {
            info.tsc_frequency()
        } else if info.numerator() != 0 && info.denominator() != 0 {
            CPUID
                .get_processor_frequency_info()
                .map(|x| x.processor_base_frequency() as u64 * 1000000)
                .map(|x| {
                    let crystal = x * info.numerator() as u64 / info.denominator() as u64;
                    crystal * info.numerator() as u64 / info.denominator() as u64
                })
        } else {
            None
        }
    })
}

const ROUNDS: u64 = 8;
const CYCLES: u64 = 500000;

fn calibrate() -> u64 {
    if let Some(x) = HIGH_PRECISION_EVENT_TIMER.as_ref() {
        let mut total = 0;
        for _ in 0..ROUNDS {
            let target = x.value() + CYCLES;
            let tsc = unsafe { _rdtsc() };
            while x.value() != target {
                core::hint::spin_loop()
            }
            total += unsafe { _rdtsc() } - tsc;
        }

        let avg = total / ROUNDS;
        let frequency = x.frequency() / CYCLES;
        frequency * avg
    } else {
        todo!()
    }
}
