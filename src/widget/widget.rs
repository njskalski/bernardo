trait Widget {
    fn focusable() -> bool;

    /*
    returns true if event was consumed.
     */
    fn on_input(input : InputEvent) -> bool;
}