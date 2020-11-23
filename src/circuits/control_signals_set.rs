#![allow(unused_imports)]
#[macro_export]
macro_rules! count_arguments {
    ($x:ident) => {
        1
    };
    ($x:ident, $($rest:ident),+) => {
        1 + wires::count_arguments!($($rest),+)
    };
}
#[macro_export]
macro_rules! generate_signal_getters {
    ($signal:ident, $($rest:ident),+) => {
        wires::generate_signal_getters!(0, $signal, $($rest),+);
    };
    ($n:expr, $signal:ident) => {
        pub fn $signal(&self) -> &wires::Wire {
            &self.signals[$n]
        }
        __concat_idents!(signal_index = $signal, _, index {
            pub fn signal_index() -> u8 {
                $n
            }
        });
    };
    ($n:expr, $signal:ident, $($rest:ident),+) => {
        wires::generate_signal_getters!($n, $signal);
        wires::generate_signal_getters!(1+$n, $($rest),+);
    };
}
#[macro_export]
macro_rules! control_signal_set {
    ($name:ident, $($signals:ident),+) => {
        control_signal_set!(wires::count_arguments!($($signals),+),$name,$($signals),+);
    };
    ($n:expr, $name:ident, $($signals:ident),+) => {
        pub struct $name {
            signals: [wires::Wire; $n],
        }

        // Sorry for polluting your namespace.
        use concat_idents::concat_idents as __concat_idents;

        #[allow(dead_code)]
        impl $name {
            pub fn new(g:&mut wires::GateGraph) -> Self {
                use std::mem::MaybeUninit;
                use std::mem::transmute;
                // I wish there was a safer way.
                // This is safe because I initialize the memory immediately afterwards.
                // https://stackoverflow.com/questions/36258417/using-a-macro-to-initialize-a-big-array-of-non-copy-elements
                // https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
                let mut signals: [MaybeUninit<wires::Wire>;$n] = unsafe { MaybeUninit::uninit().assume_init() };
                for elem in &mut signals[..] {
                    // TODO per wire names.
                    *elem = MaybeUninit::new(wires::Wire::new(g,stringify!($name)));
                }
                Self {
                    signals: unsafe{ transmute(signals) }
                }
            }
            pub fn len() -> usize {
                $n
            }
            pub fn connect(&mut self, g: &mut wires::GateGraph, input: &[wires::GateIndex; $n]) {
                for (signal, input) in self.signals.iter_mut().zip(input) {
                    signal.connect(g, *input)
                }
            }
            wires::generate_signal_getters!($($signals),+);
        }
    };
}
#[macro_export]
macro_rules! signals_to_bits {
    ($signal_set:ty) => {
        0
    };
    ($signal_set:ty, $($signals:ident),+) => {
        {
            use concat_idents::concat_idents;
            wires::signals_to_bits!(0, $signal_set, $($signals),+)
        }
    };
    ($bits:expr, $signal_set:ty, $signal:ident) => {
        concat_idents!(signal_index = $signal, _, index {
            ($bits | (1 << $signal_set::signal_index()))
        });
    };
    ($bits:expr, $signal_set:ty, $signal:ident, $($rest:ident),+) => {
        wires::signals_to_bits!(wires::signals_to_bits!($bits,$signal_set, $signal), $signal_set, $($rest),+);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as wires;

    control_signal_set!(TestSignals, s1, s2, s3);

    #[test]
    fn test_set() {
        assert_eq!(TestSignals::len(), 3);
        assert_eq!(TestSignals::s1_index(), 0);
        assert_eq!(TestSignals::s2_index(), 1);
        assert_eq!(TestSignals::s3_index(), 2);
    }

    #[test]
    fn test_signals_to_bits() {
        assert_eq!(signals_to_bits!(TestSignals), 0);
        assert_eq!(signals_to_bits!(TestSignals, s1), 1);
        assert_eq!(signals_to_bits!(TestSignals, s3), 0b100);

        assert_eq!(signals_to_bits!(TestSignals, s2, s3), 0b110);
        assert_eq!(signals_to_bits!(TestSignals, s3, s2), 0b110);
        assert_eq!(signals_to_bits!(TestSignals, s1, s2, s3), 0b111);
    }
}
