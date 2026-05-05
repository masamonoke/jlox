pub type Number = f32;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Number(Number),
    String(String),
    Nil,
}
