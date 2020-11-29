use std::time::Instant;

pub struct ClockTimer {
    t: Instant,
    m_avg: u64,
    print_period: u64,
    clock_cycles: u64,
}
impl ClockTimer {
    pub fn new(print_period: u64) -> ClockTimer {
        ClockTimer {
            t: Instant::now(),
            m_avg: 0,
            print_period,
            clock_cycles: 0,
        }
    }
    pub fn clock(&mut self) {
        let elapsed = self.t.elapsed().as_nanos() as u64;
        self.t = Instant::now();
        self.clock_cycles += 1;
        self.m_avg = (self.m_avg * (self.clock_cycles - 1) + elapsed) / self.clock_cycles;
        if self.clock_cycles % self.print_period == 0 {
            println!("{}ns/clock avg", self.m_avg);
        }
    }
}
impl Drop for ClockTimer {
    fn drop(&mut self) {
        println!("\n{}ns/clock avg", self.m_avg);
    }
}
