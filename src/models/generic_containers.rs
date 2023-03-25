use serde::{Serialize, Serializer, ser::SerializeStruct};

#[derive(Serialize)]
pub struct IDContainer {
    pub id: u32,
    pub name: String
}

#[derive(Serialize)]
pub struct BasicContainer {
    pub name: String,
    pub description: String
}

#[derive(Serialize)]
pub struct Container {
    pub id: u32,
    pub name: String,
    pub description: String
}

pub struct ContainerData<const N: usize, C: serde::Serialize = Container> {
    pub parents: [IDContainer; N],
    pub container: BasicContainer,
    pub children: Vec<C>
}

macro_rules! impl_serial_for_data {
    ($n:literal) => {
        impl <C: Serialize> Serialize for ContainerData<$n, C> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
                let mut state = Serializer::serialize_struct(serializer, "Data", 3)?;
                state.serialize_field("parents", &self.parents)?;
                state.serialize_field("container", &self.container)?;
                state.serialize_field("children", &self.children)?;
                state.end()
            }
        }
    };
}

impl_serial_for_data!(0);
impl_serial_for_data!(1);
impl_serial_for_data!(2);
