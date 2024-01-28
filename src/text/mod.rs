pub mod buffer_state;
pub mod text_buffer;

mod buffer_state_test;
mod contents_and_cursors;

#[cfg(test)]
mod rope_tests;

#[cfg(feature = "arbitrary")]
mod buffer_state_fuzz;
