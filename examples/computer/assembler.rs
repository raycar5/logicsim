use super::instruction_set::{Instruction, InstructionType};
use auto_from::From;
pub use logicsim::data_structures::BitIter;
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum PointerType {
    RAM,
    ROM,
}
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Pointer(pub u8, pub PointerType);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Label(pub usize);
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct InstructionWithPtr {
    ty: InstructionType,
    ptr: Pointer,
}
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct InstructionWithLabel {
    ty: InstructionType,
    label: Label,
}
pub trait IntoInstructionWithPointer {
    fn with_ptr(self, ptr: Pointer) -> InstructionWithPtr;
}
impl IntoInstructionWithPointer for InstructionType {
    fn with_ptr(self, ptr: Pointer) -> InstructionWithPtr {
        InstructionWithPtr { ty: self, ptr }
    }
}
pub trait IntoInstructionWithLabel {
    fn with_label(self, label: Label) -> InstructionWithLabel;
}
impl IntoInstructionWithLabel for InstructionType {
    fn with_label(self, label: Label) -> InstructionWithLabel {
        InstructionWithLabel { ty: self, label }
    }
}
#[derive(From, Debug)]
pub enum Directive {
    Instruction(Instruction),
    InstructionWithPtr(InstructionWithPtr),
    InstructionWithLabel(InstructionWithLabel),
    Data(u16),
}
impl From<InstructionType> for Directive {
    fn from(ty: InstructionType) -> Self {
        ty.with_0().into()
    }
}

pub fn byte_iter_to_directives<I: Iterator<Item = u8>>(iter: I) -> Vec<Directive> {
    let mut out = Vec::new();
    for bytes in iter.collect::<Vec<_>>().chunks(2) {
        let word = bytes[0] as u16 | ((bytes.get(1).copied().unwrap_or(0) as u16) << 8);
        out.push(Directive::Data(word))
    }
    out
}

macro_rules! assemble_inner {
    ($vec:ident, $labels:ident, label $label:ident; $($rest:tt)*) => {
        let $label = Label($labels.len());
        $labels.push(0);
        assemble_inner!($vec, $labels, $($rest)*);
    };
    ($vec:ident, $labels:ident, data#$label:ident : $val:expr; $($rest:tt)*) => {
        $labels[$label.0] = ($vec.len() * 2) as u8;
        $vec.append(&mut byte_iter_to_directives($val));
        assemble_inner!($vec, $labels, $($rest)*);
    };
    ($vec:ident, $labels:ident, $label:ident : $val:expr; $($rest:tt)*) => {
        $labels[$label.0] = ($vec.len() * 2) as u8;
        $vec.push($val.into());
        assemble_inner!($vec, $labels, $($rest)*);
    };
    ($vec:ident, $labels:ident, $ptr:ident =ram= $val:expr; $($rest:tt)*) => {
        let $ptr = Pointer($val, PointerType::RAM);
        assemble_inner!($vec, $labels, $($rest)*);
    };
    ($vec:ident, $labels:ident, $ptr:ident =rom= $val:expr; $($rest:tt)*) => {
        let $ptr = Pointer($val, PointerType::ROM);
        assemble_inner!($vec, $labels, $($rest)*);
    };
    ($vec:ident, $labels:ident, $val:expr; $($rest:tt)*) => {
        $vec.push($val.into());
        assemble_inner!($vec, $labels, $($rest)*);
    };
    ($vec:ident, $labels:ident, ) => { };

}
macro_rules! assemble {
    ($($all:tt)*) => {
        {
            let mut directives = Vec::<Directive>::new();
            #[allow(unused_mut)]
            let mut labels = Vec::<u8>::new();
            assemble_inner!(directives, labels, $($all)*);
            assemble(directives, labels)
        }
    };
}
pub fn assemble(directives: Vec<Directive>, labels: Vec<u8>) -> Vec<u16> {
    let ram_mask = 1u8 << 7;
    let mut out = Vec::new();
    for directive in directives {
        match directive {
            Directive::Instruction(instruction) => out.push(instruction.into()),
            Directive::InstructionWithPtr(InstructionWithPtr { ty, ptr }) => out.push(
                Instruction {
                    ty,
                    data: ptr.0
                        | if matches!(ptr.1, PointerType::RAM) {
                            ram_mask
                        } else {
                            0
                        },
                }
                .into(),
            ),
            Directive::InstructionWithLabel(InstructionWithLabel { ty, label }) => out.push(
                Instruction {
                    ty,
                    data: labels[label.0],
                }
                .into(),
            ),
            Directive::Data(data) => out.push(data),
        }
    }
    assert!(
        out.len() * 2 <= 128,
        "Your program is too big! len:{}",
        out.len() * 2
    );
    out
}
