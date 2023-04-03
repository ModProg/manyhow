use example::*;
fn attr() {
    #[attr_emit]
    struct Struct;
    output();
}
fn function() {
    emit!(
        struct Struct;
    );
    output();
}
fn derive() {
    #[derive(Emit)]
    struct Struct;
    _ = Struct;
    output();
}

fn main() {}
