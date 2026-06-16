from dataclasses import dataclass

STANDARD_TYPE_ASSOC : dict[str,str] ={
    "i8":"number",
    "i16":"number",
    "i32":"number",
    "i64":"string",
    "isize":"number",
    "u8":"number",
    "u16":"number",
    "u32":"number",
    "u64":"number",
    "f32":"number",
    "f64":"number",
    "usize":"number",    
    "str": "string",
    "String": "string"
}