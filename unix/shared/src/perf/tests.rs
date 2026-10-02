#[cfg(perf_tracking)]
use super::{AtomicCycleAddTimer, CycleCounter};
use super::{
    CommandBufferRole, GpuBusy, PipelineTimings, ShaderTimings, SubmitTimings, TimingOutput,
};
#[cfg(perf_tracking)]
use crate::tsc::rdtsc;

#[test]
fn timing_output_layout_and_fallback_are_stable() {
    assert_eq!(size_of::<TimingOutput<ShaderTimings>>(), 24);
    assert_eq!(
        align_of::<TimingOutput<ShaderTimings>>(),
        align_of::<ShaderTimings>()
    );
    assert_eq!(size_of::<TimingOutput<PipelineTimings>>(), 16);
    let value = TimingOutput::<ShaderTimings>::new().into_inner();
    assert_eq!(
        (value.preparation_ns, value.library_ns, value.function_ns),
        (0, 0, 0)
    );
}

#[test]
fn timing_output_publishes_only_in_perf_builds() {
    let mut output = TimingOutput::new();
    output.write(PipelineTimings {
        preparation_ns: 12,
        build_ns: 34,
    });
    let value = output.into_inner();
    let expected = if cfg!(perf_tracking) {
        (12, 34)
    } else {
        (0, 0)
    };
    assert_eq!((value.preparation_ns, value.build_ns), expected);
}

/// The submit timings keep one layout for the i686 PE caller and the 64-bit handler.
#[test]
fn submit_timings_layout_is_stable() {
    assert_eq!(size_of::<GpuBusy>(), 16);
    assert_eq!(align_of::<SubmitTimings>(), 8);
    assert_eq!(size_of::<SubmitTimings>(), 72);
    assert_eq!(core::mem::offset_of!(SubmitTimings, gpu), 24);
    let timings = SubmitTimings::new();
    let present = &timings.gpu[CommandBufferRole::Present as usize];
    assert_eq!((present.ns, present.buffers), (0, 0));
}

/// A counter keeps its value across `add` and `load`, and `take` returns it and leaves zero.
#[cfg(perf_tracking)]
#[test]
fn cycle_counter_adds_loads_and_takes() {
    let counter = CycleCounter::new();
    assert_eq!(counter.load(), 0);
    counter.add(5);
    counter.add(7);
    assert_eq!(counter.load(), 12);
    assert_eq!(counter.take(), 12);
    assert_eq!(counter.load(), 0);
    assert_eq!(counter.take(), 0);
    counter.add(u64::MAX);
    counter.add(2);
    assert_eq!(counter.take(), 1, "adds wrap like the plain accumulators");
}

/// A drain racing the adds neither loses nor repeats one.
///
/// What the drains took plus what is left is exactly every add.
#[cfg(perf_tracking)]
#[test]
fn cycle_counter_take_races_adds_without_losing_any() {
    const ADDERS: u64 = 8;
    const ADDS: u64 = 50_000;
    let counter = CycleCounter::new();
    let finished = std::sync::atomic::AtomicU64::new(0);
    let mut taken = 0;
    std::thread::scope(|scope| {
        for _ in 0..ADDERS {
            scope.spawn(|| {
                for _ in 0..ADDS {
                    counter.add(1);
                }
                finished.fetch_add(1, std::sync::atomic::Ordering::Release);
            });
        }
        while finished.load(std::sync::atomic::Ordering::Acquire) < ADDERS {
            taken += counter.take();
        }
    });
    taken += counter.take();
    assert_eq!(taken, ADDERS * ADDS);
}

/// The timer books the elapsed counter ticks into its target when it drops.
#[cfg(perf_tracking)]
#[test]
fn atomic_cycle_timer_books_elapsed_ticks_on_drop() {
    let counter = CycleCounter::new();
    let timer = AtomicCycleAddTimer::start(Some(&counter));
    let began = rdtsc();
    while rdtsc() == began {
        std::hint::spin_loop();
    }
    assert_eq!(counter.load(), 0, "nothing lands before the drop");
    drop(timer);
    assert!(counter.load() >= 1);
    let first = counter.load();
    drop(AtomicCycleAddTimer::start(Some(&counter)));
    assert!(
        counter.load() >= first,
        "a second timer adds to the same counter"
    );
}

/// A timer with no target reads no clock and writes nothing.
#[cfg(perf_tracking)]
#[test]
fn atomic_cycle_timer_without_a_target_is_inert() {
    let timer = AtomicCycleAddTimer::start(None);
    assert_eq!(timer.start, 0, "no clock read for a disabled timer");
    drop(timer);
}
