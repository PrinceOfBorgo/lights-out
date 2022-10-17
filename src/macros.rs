#[macro_export]
macro_rules! param_textbox {
    ($field: ident) => {
       ValueTextBox::new(TextBox::new(), ParseFormatter::new())
            .validate_while_editing(false)
            .lens(Params::$field)
            .controller(ParamsController {})
            .align_left()
    };
}