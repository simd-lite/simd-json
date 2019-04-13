use std::fs::File;
use std::io::Read;
use std::iter;

#[cfg(feature = "perf")]
fn bench(name: &str) {
    use perfcnt::linux::{HardwareEventType, PerfCounterBuilderLinux};
    use perfcnt::{AbstractPerfCounter, PerfCounter};

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

    let mut data_entries: Vec<Vec<u8>> = iter::repeat(vec).take(120).collect();
    // Run some warmup;

    for mut bytes in &mut data_entries[..20] {
        simdjson::to_borrowed_value(&mut bytes).unwrap();
    }
    let mut cycles: u64 = 0;
    let mut instructions: u64 = 0;
    let mut cache_misses: u64 = 0;
    let mut cache_references: u64 = 0;
    let mut branch_instructions: u64 = 0;
    for mut bytes in &mut data_entries[20..] {
        // Set up counters
        let mut cr = pc(HardwareEventType::CacheReferences);
        let mut cm = pc(HardwareEventType::CacheMisses);
        let mut inst = pc(HardwareEventType::Instructions);
        let mut bi = pc(HardwareEventType::BranchInstructions);
        let mut cc = pc(HardwareEventType::CPUCycles);

        // run the measurement
        let r = simdjson::to_borrowed_value(&mut bytes);
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
        cache_references += cr.read().unwrap();
        cache_misses += cm.read().unwrap();
        cycles += cc.read().unwrap();
        instructions += inst.read().unwrap();
        branch_instructions += bi.read().unwrap();
    }
    println!();
    println!("============[{:^16}]============", name);
    println!("  => Cycles:             {:15}", cycles / 100);
    println!("  => Instructions:       {:15}", instructions / 100);
    println!("  => BranchInstructions: {:15}", branch_instructions / 100);
    println!("  => CacheMisses:        {:15}", cache_misses / 100);
    println!("  => CacheReferences:    {:15}", cache_references / 100);
    println!("==========================================");

    /*
    let start = Instant::now();
    pc.start();
    let output = routine(input);
    simdjson::to_borrowed_value(&mut bytes).unwrap();
    pc.stop();
    self.elapsed += start.elapsed();
    self.perf.cycles += pc.read().unwrap();
    */
}

#[cfg(not(feature = "perf"))]
fn bench(_name: &str) {
    println!("Perf requires linux to run and the perf feature to be enabled")
}

fn main() {
    bench("apache_builds");
    bench("canada");
    bench("citm_catalog");
    bench("log");
    bench("twitter");
}
