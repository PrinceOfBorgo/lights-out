#[macro_export]
macro_rules! nonzero_textbox {
    ($field: ident) => {
        ValueTextBox::new(TextBox::new(), NonZeroFormatter)
            .validate_while_editing(false)
            .lens(Params::$field)
            .controller(ParamsController {})
            .align_left()
    };
}
