#[macro_export]
macro_rules! ser_enum {
    (
        $name: ident, $serializer: ident
    ) => {
        // fn ser_enum(name: &syn::DeriveInput.ident, item: syn::Data::Enum, serializer: Serializer) -> Result<Vec<u8>> {
        //     match 
        // }
        let mut i: usize = 0;
        match *self {
            $(
                #name::enum_item.ident => $serializer.serialize_unit_variant(#name, increment(&mut i)-1, enum_item.ident),
            )+
        }             
    }
}