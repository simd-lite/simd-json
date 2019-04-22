#[cfg(feature = "perf")]
fn bench(name: &str) {
    use perfcnt::linux::{HardwareEventType, PerfCounterBuilderLinux};
    use perfcnt::{AbstractPerfCounter, PerfCounter};
    use std::fs::File;
    use std::io::Read;
    use std::iter;

    fn pc(event_type: HardwareEventType) -> PerfCounter {
        PerfCounterBuilderLinux::from_hardware_event(event_type)
            .for_pid(0)
            .exclude_kernel()
            .exclude_idle()
            .finish()
            .unwrap()
    }

    let mut vec = Vec::new();
    let mut f = String::from("data/");
    f.push_str(name);
    f.push_str(".json");
    File::open(f).unwrap().read_to_end(&mut vec).unwrap();
    let bytes = vec.len();
    let rounds: u64 = 1000;
    let warmup: u64 = 200;
    let mut data_entries: Vec<Vec<u8>> = iter::repeat(vec).take((rounds + warmup) as usize).collect();
    // Run some warmup;

    for mut bytes in &mut data_entries[..warmup as usize] {
        simd_json::to_borrowed_value(&mut bytes).unwrap();
    }
    let mut cycles_avg: u64 = 0;
    let mut cycles_top: u64 = 0;
    let mut instructions_avg: u64 = 0;
    //let mut instructions_top: u64 = 0;
    let mut cache_misses_avg: u64 = 0;
    let mut cache_references_avg: u64 = 0;
    let mut branch_instructions_avg: u64 = 0;
    for mut bytes in &mut data_entries[warmup as usize..] {
        // Set up counters
        let mut cr = pc(HardwareEventType::CacheReferences);
        let mut cm = pc(HardwareEventType::CacheMisses);
        let mut inst = pc(HardwareEventType::Instructions);
        let mut bi = pc(HardwareEventType::BranchInstructions);
        let mut cc = pc(HardwareEventType::CPUCycles);

        // run the measurement
        let r = simd_json::to_borrowed_value(&mut bytes);
        // Stop counters
        cr.stop().unwrap();
        cm.stop().unwrap();
        cc.stop().unwrap();
        inst.stop().unwrap();
        bi.stop().unwrap();
        // we make sure that dropping doesn't happen untill we are done with our counting.
        // better safe then sorry.
        assert!(r.is_ok());
        // do our accounting
        let c = cc.read().unwrap();
        if c < cycles_top || cycles_top == 0 {
            cycles_top = c;
        }
        cycles_avg += c;
        instructions_avg += inst.read().unwrap();
        branch_instructions_avg += bi.read().unwrap();
        cache_references_avg += cr.read().unwrap();
        cache_misses_avg += cm.read().unwrap();
    }
    //    println!();
    //    println!("============[{:^16}]============", name);
    cycles_avg /= rounds;
    cache_references_avg /= rounds;
    cache_misses_avg /= rounds;
    instructions_avg /= rounds;
    branch_instructions_avg /= rounds;

    println!(
        "{:14} {:10} {:10} {:10} {:10} {:10} {:10.3} {:10.3}",
        name,
        cycles_avg,
        instructions_avg,
        branch_instructions_avg,
        cache_misses_avg,
        cache_references_avg,
        ((cycles_top as f64) / bytes as f64),
        ((cycles_avg as f64) / bytes as f64)
    );
}

#[cfg(not(feature = "perf"))]
fn bench(_name: &str) {
    println!("Perf requires linux to run and the perf feature to be enabled")
}

fn main() {
    println!(
        "{:^14} {:^10} {:^21} {:^21} {:^21}",
        " ", "", "Instructions", "Cache.", "Cycle/byte"
    );
    println!(
        "{:^14} {:^10} {:^10} {:^10} {:^10} {:^10} {:^10} {:^10}",
        "Name", "Cycles", "Normal.", "Branch", "Misses", "References", "Best", "Avg"
    );
    bench("apache_builds");
    bench("canada");
    bench("citm_catalog");
    bench("github_events");
    bench("gsoc-2018");
    bench("instruments");
    bench("log");
    bench("marine_ik");
    bench("mesh");
    bench("mesh.pretty");
    bench("numbers");
    bench("random");
    bench("twitter");
    bench("twitterescaped");
    bench("update-center");
}
