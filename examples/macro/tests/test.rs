use example::*;

#[test]
fn attr() {
    #[attr_item_as_dummy]
    struct ItemAsDummy;
    _ = ItemAsDummy;

    #[attr_no_dummy]
    struct ItemAsDummy; // does not conflict with above

    #[attr_custom_dummy]
    struct ItemAsDummy; // does not conflict with above
    dummy();
}

#[test]
fn function() {
    input_as_dummy!(
        struct InputAsDummy;
    );
    _ = InputAsDummy;
    no_dummy!(
        // does not conflict with above
        struct InputAsDummy;
    );
    custom_dummy!(
        // does not conflict with above
        struct InputAsDummy;
    );
    dummy();
}

#[test]
fn derive() {
    #[derive(NoDummy)]
    struct NoDummy;
    _ = NoDummy;
    #[derive(Dummy)]
    struct Dummy;
    _ = Dummy;
    dummy();
}
