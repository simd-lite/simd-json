use std::env;

#[cfg(all(feature = "perf", target_arch = "x86_64", target_os = "linux"))]
mod int {
    const ROUNDS: usize = 2000;
    const WARMUP: usize = 200;
    use colored::*;
    use perfcnt::linux::{HardwareEventType, PerfCounterBuilderLinux};
    use perfcnt::{AbstractPerfCounter, PerfCounter};
    use serde::{Deserialize, Serialize};
    use simd_json::{Deserializer, Implementation};
    use std::io::BufReader;

    #[derive(Default, Serialize, Deserialize)]
    struct Stats {
        algo: String,
        best: Stat,
        total: Stat,
        iters: u64,
    }
    impl Stats {
        fn new(algo: Implementation) -> Self {
            Stats {
                algo: algo.to_string(),
                ..Default::default()
            }
        }
    }

    #[derive(Default, Serialize, Deserialize)]
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
            if self.best.branch_instructions > branch_instructions
                || self.best.branch_instructions == 0
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
                "{:20} {:10} {:10} {:10} {:10} {:10} {:10.3} {:10.3} {:21}",
                name,
                cycles,
                instructions,
                branch_instructions,
                cache_misses,
                cache_references,
                ((self.best.cycles as f64) / bytes as f64),
                ((cycles as f64) / bytes as f64),
                self.algo
            );
        }
        pub fn print_diff(&self, baseline: &Stats, name: &str, bytes: usize) {
            let cycles = self.total.cycles / self.iters;
            let instructions = self.total.instructions / self.iters;
            let cache_misses = self.total.cache_misses / self.iters;
            let cache_references = self.total.cache_references / self.iters;
            let branch_instructions = self.total.branch_instructions / self.iters;
            let best_cycles_per_byte = (self.best.cycles as f64) / bytes as f64;
            let cycles_per_byte = cycles as f64 / bytes as f64;

            let cycles_b = baseline.total.cycles / baseline.iters;
            let instructions_b = baseline.total.instructions / baseline.iters;
            let cache_misses_b = baseline.total.cache_misses / baseline.iters;
            let cache_references_b = baseline.total.cache_references / baseline.iters;
            let branch_instructions_b = baseline.total.branch_instructions / baseline.iters;
            let best_cycles_per_byte_b = (baseline.best.cycles as f64) / bytes as f64;
            let cycles_per_byte_b = cycles_b as f64 / bytes as f64;

            fn d(d: f64) -> String {
                if d < 1.0 && d > -1.0 {
                    format!("{:9.3}%", d).yellow().to_string()
                } else if d >= 1.0 {
                    format!("{:9.3}%", d).red().to_string()
                } else {
                    format!("{:9.3}%", d).green().to_string()
                }
            }

            println!(
                "{:20} {:>10} {:>10} {:>10} {:>10} {:>10} {:10} {:10} {:21}",
                format!("{}(+/-)", name),
                d((1.0 - cycles_b as f64 / cycles as f64) * 100.0),
                d((1.0 - instructions_b as f64 / instructions as f64) * 100.0),
                d((1.0 - branch_instructions_b as f64 / branch_instructions as f64) * 100.0),
                d((1.0 - cache_misses_b as f64 / cache_misses as f64) * 100.0),
                d((1.0 - cache_references_b as f64 / cache_references as f64) * 100.0),
                d((1.0 - best_cycles_per_byte_b as f64 / best_cycles_per_byte as f64) * 100.0),
                d((1.0 - cycles_per_byte_b as f64 / cycles_per_byte as f64) * 100.0),
                baseline.algo
            );
        }
    }

    pub fn bench(name: &str, baseline: bool) {
        use std::fs::{self, File};
        use std::io::Read;
        use std::iter;

        let mut vec = Vec::new();
        let mut f = String::from("data/");
        f.push_str(name);
        f.push_str(".json");
        File::open(f).unwrap().read_to_end(&mut vec).unwrap();
        let bytes = vec.len();
        let mut data_entries: Vec<Vec<u8>> =
            iter::repeat(vec).take((ROUNDS + WARMUP) as usize).collect();
        // Run some warmup;

        for mut bytes in &mut data_entries[..WARMUP as usize] {
            simd_json::to_borrowed_value(&mut bytes).unwrap();
        }
        let mut stats = Stats::new(Deserializer::algorithm());
        for mut bytes in &mut data_entries[WARMUP as usize..] {
            // Set up counters
            let pc = stats.start();

            // run the measurement
            let r = simd_json::to_borrowed_value(&mut bytes);
            // Stop counters
            stats.stop(pc);
            // we make sure that dropping doesn't happen until we are done with our counting.
            // better safe then sorry.
            assert!(r.is_ok());
            // do our accounting
        }
        stats.print(name, bytes);
        if baseline {
            let _ = fs::create_dir(".baseline");
            fs::write(
                format!(".baseline/{}.json", name),
                serde_json::to_vec(&stats).expect("Failed to serialize"),
            )
            .expect("Unable to write file");
        } else {
            let _ = fs::create_dir(".current");
            fs::write(
                format!(".current/{}.json", name),
                serde_json::to_vec(&stats).expect("Failed to serialize"),
            )
            .expect("Unable to write file");
            let file =
                File::open(format!(".baseline/{}.json", name)).expect("Could not open baseline");
            let reader = BufReader::new(file);
            let baseline: Stats = serde_json::from_reader(reader).expect("Failed to read baseline");
            stats.print_diff(&baseline, name, bytes);
        }
    }
}

#[cfg(not(all(feature = "perf", target_arch = "x86_64", target_os = "linux")))]
mod int {
    pub fn bench(_name: &str, _baseline: bool) {
        if std::env::consts::ARCH != "x86_64" {
            println!("This Perf example only supports x86_64 for now.");
        } else if std::env::consts::OS != "linux" {
            println!("Perf requires linux to run and the perf feature to be enabled");
        }
    }
}

fn main() {
    // e.g.: cargo run --release --example perf --features perf -- --baseline
    let mut opts = getopts::Options::new();
    opts.optflag("b", "baseline", "create baseline");
    let args: Vec<String> = env::args().collect();
    let matches = opts.parse(&args[1..]).unwrap();

    println!(
        "{:^20} {:^10} {:^21} {:^21} {:^21} {:21}",
        " ", "", "Instructions", "Cache.", "Cycle/byte", "Algorithm"
    );
    println!(
        "{:^20} {:^10} {:^10} {:^10} {:^10} {:^10} {:^10} {:^10}",
        "Name", "Cycles", "Normal.", "Branch", "Misses", "References", "Best", "Avg"
    );

    let baseline = matches.opt_present("b");
    int::bench("apache_builds", baseline);
    int::bench("canada", baseline);
    int::bench("citm_catalog", baseline);
    int::bench("github_events", baseline);
    int::bench("gsoc-2018", baseline);
    int::bench("instruments", baseline);
    int::bench("log", baseline);
    int::bench("marine_ik", baseline);
    int::bench("mesh", baseline);
    int::bench("mesh.pretty", baseline);
    int::bench("numbers", baseline);
    int::bench("random", baseline);
    int::bench("twitter", baseline);
    int::bench("twitterescaped", baseline);
    int::bench("update-center", baseline);
}
