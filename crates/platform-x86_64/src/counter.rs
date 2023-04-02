use crate::devices;
use crate::CPUID;
use log::info;
use common::sync::SyncLazy;

static COUNTER: SyncLazy<&'static dyn Counter> = SyncLazy::new(find_counter);

pub trait Counter: Send + Sync {
    fn frequency(&self) -> u64;
    fn calibrate(&self);
}

pub fn init() {
    SyncLazy::force(&COUNTER);
}

pub fn frequency() -> u64 {
    COUNTER.frequency()
}

pub fn calibrate() {
    COUNTER.calibrate();
}

fn find_counter() -> &'static dyn Counter {
    let has_tsc = CPUID
        .get_feature_info()
        .map(|x| x.has_tsc())
        .unwrap_or_default();

    if has_tsc {
        info!("Counter: TSC");
        devices::TIME_STAMP_COUNTER.as_ref()
    } else if let Some(hpet) = devices::HIGH_PRECISION_EVENT_TIMER.as_ref() {
        info!("Counter: HPET");
        hpet
    } else {
        panic!("No counter found");
    }
}
