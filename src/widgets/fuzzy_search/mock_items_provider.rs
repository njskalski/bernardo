pub mod mock {
    use std::borrow::Cow;

    use streaming_iterator::StreamingIterator;

    use crate::AnyMsg;
    use crate::primitives::alphabet::mock::ALPHABET;
    use crate::widgets::fuzzy_search::helpers::is_subsequence;
    use crate::widgets::fuzzy_search::item_provider::{FuzzyItem, FuzzyItemsProvider};
    use crate::widgets::main_view::msg::MainViewMsg;

    pub struct MockItemProvider {
        num_items: usize,
        items: Vec<String>,
    }

    impl MockItemProvider {
        pub fn new(num_items: usize) -> Self {
            let mut items: Vec<String> = vec![];

            for i in 0..num_items {
                let mut idx = i;
                let mut item = String::default();

                loop {
                    if !item.is_empty() {
                        item += " ";
                    }

                    item += ALPHABET[idx % ALPHABET.len()];
                    idx /= ALPHABET.len();
                    if idx == 0 {
                        break;
                    }
                }

                items.push(item);
            }

            MockItemProvider {
                num_items,
                items,
            }
        }
    }

    impl FuzzyItem for String {
        fn display_name(&self) -> Cow<str> {
            self.into()
        }

        fn on_hit(&self) -> Box<dyn AnyMsg> {
            Box::new(MainViewMsg::ClozeHover)
        }
    }

    impl FuzzyItemsProvider for MockItemProvider {
        fn context_name(&self) -> &str {
            "mock"
        }

        fn items(&self, query: String, limit: usize) -> Box<dyn StreamingIterator<Item=Box<dyn FuzzyItem>>> {
            Box::new(self.items.iter().filter(move |t| is_subsequence(t, &query)).take(limit).map(|f| Box::new(f.to_string()) as Box<dyn FuzzyItem>))
        }
    }
}