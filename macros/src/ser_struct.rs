#[macro_export]
macro_rules! ser_struct {
    (
        $idents: ident
    ) => {
        $(
            s.serialize_field($idents[increment(j)-1], #name.$idents[j-1])
        )+
    }
}