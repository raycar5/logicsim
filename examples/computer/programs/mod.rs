mod greeter;
mod hello_world;
mod multiply;

pub enum OutputType {
    Text,
    Number,
}
pub trait Program {
    fn output_type(&self) -> OutputType;
    fn ram_address_space_bits(&self) -> usize;
    fn clock_print_interval(&self) -> u64;
    fn rom(&self) -> Vec<u16>;
}
pub fn list_programs() -> Vec<&'static str> {
    vec!["greeter", "hello_world", "multiply"]
}
// I'll forgive myself for using dynamic dispatch on this one.
pub fn program(name: &str) -> Option<Box<dyn Program>> {
    Some(match name {
        "greeter" => Box::new(greeter::Greeter()),
        "hello_world" => Box::new(hello_world::HelloWorld()),
        "multiply" => Box::new(multiply::Multiply()),
        _ => return None,
    })
}
