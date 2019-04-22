use perfcnt::linux::{HardwareEventType, PerfCounterBuilderLinux};
use perfcnt::{AbstractPerfCounter, PerfCounter};
use serde;
use serde_derive;
use serde_json;

#[derive(Default)]
struct Stats {
    best: Stat,
    total: Stat,
    iters: u64,
}

#[derive(Default)]
struct Stat {
    cycles: u64,
    instructions: u64,
    cache_misses: u64,
    cache_references: u64,
    branch_instructions: u64,
}

struct Counter {
    cycles: PerfCounter,
    instructions: PerfCounter,
    cache_misses: PerfCounter,
    cache_references: PerfCounter,
    branch_instructions: PerfCounter,
}

impl Stats {
    pub fn start(&self) -> Counter {
        fn pc(event_type: HardwareEventType) -> PerfCounter {
            PerfCounterBuilderLinux::from_hardware_event(event_type)
                .for_pid(0)
                .exclude_kernel()
                .exclude_idle()
                .finish()
                .unwrap()
        }
        Counter {
            cycles: pc(HardwareEventType::CPUCycles),
            instructions: pc(HardwareEventType::Instructions),
            cache_misses: pc(HardwareEventType::CacheMisses),
            cache_references: pc(HardwareEventType::CacheReferences),
            branch_instructions: pc(HardwareEventType::BranchInstructions),
        }
    }

    pub fn stop(&mut self, mut counter: Counter) {
        counter.cycles.stop().unwrap();
        counter.instructions.stop().unwrap();
        counter.cache_misses.stop().unwrap();
        counter.cache_references.stop().unwrap();
        counter.branch_instructions.stop().unwrap();
        self.iters += 1;
        let cycles = counter.cycles.read().unwrap();
        let instructions = counter.instructions.read().unwrap();
        let cache_misses = counter.cache_misses.read().unwrap();
        let cache_references = counter.cache_references.read().unwrap();
        let branch_instructions = counter.branch_instructions.read().unwrap();
        self.total.cycles += cycles;
        self.total.instructions += instructions;
        self.total.cache_misses += cache_misses;
        self.total.cache_references += cache_references;
        self.total.branch_instructions += branch_instructions;
        if self.best.cycles > cycles || self.best.cycles == 0 {
            self.best.cycles = cycles
        };
        if self.best.instructions > instructions || self.best.instructions == 0 {
            self.best.instructions = instructions
        };
        if self.best.cache_misses > cache_misses || self.best.cache_misses == 0 {
            self.best.cache_misses = cache_misses
        };
        if self.best.cache_references > cache_references || self.best.cache_references == 0 {
            self.best.cache_references = cache_references
        };
        if self.best.branch_instructions > branch_instructions || self.best.branch_instructions == 0
        {
            self.best.branch_instructions = branch_instructions
        };
    }
    pub fn print(&self, name: &str, bytes: usize) {
        let cycles = self.total.cycles / self.iters;
        let instructions = self.total.instructions / self.iters;
        let cache_misses = self.total.cache_misses / self.iters;
        let cache_references = self.total.cache_references / self.iters;
        let branch_instructions = self.total.branch_instructions / self.iters;

        println!(
            "{:14} {:10} {:10} {:10} {:10} {:10} {:10.3} {:10.3}",
            name,
            cycles,
            instructions,
            branch_instructions,
            cache_misses,
            cache_references,
            ((self.best.cycles as f64) / bytes as f64),
            ((cycles as f64) / bytes as f64)
        );
    }
}

#[cfg(feature = "perf")]
fn bench(name: &str) {
    use std::fs::File;
    use std::io::Read;
    use std::iter;

    let mut vec = Vec::new();
    let mut f = String::from("data/");
    f.push_str(name);
    f.push_str(".json");
    File::open(f).unwrap().read_to_end(&mut vec).unwrap();
    let bytes = vec.len();
    let rounds: u64 = 1000;
    let warmup: u64 = 200;
    let mut data_entries: Vec<Vec<u8>> =
        iter::repeat(vec).take((rounds + warmup) as usize).collect();
    // Run some warmup;

    for mut bytes in &mut data_entries[..warmup as usize] {
        simd_json::to_borrowed_value(&mut bytes).unwrap();
    }
    let mut stats = Stats::default();
    for mut bytes in &mut data_entries[warmup as usize..] {
        // Set up counters
        let pc = stats.start();

        // run the measurement
        let r = simd_json::to_borrowed_value(&mut bytes);
        // Stop counters
        stats.stop(pc);
        // we make sure that dropping doesn't happen untill we are done with our counting.
        // better safe then sorry.
        assert!(r.is_ok());
        // do our accounting
    }
    stats.print(name, bytes);
    //    println!();
    //    println!("============[{:^16}]============", name);
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
